#![allow(dead_code)]

use crate::{binary, bits::{self, Uint128}, subtle};

// Element represents an element of the field GF(2^255-19).
// An element is represented as a radix-2^51 value.
// An element t represents the integer
//     t.0 + t.1*2^51 + t.2*2^102 + t.3*2^153 + t.4*2^204
// Between operations, all limbs are expected to be lower than 2^52.
// The zero value is a valid zero element.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Element(
    pub(crate) u64,
    pub(crate) u64,
    pub(crate) u64,
    pub(crate) u64,
    pub(crate) u64,
);

impl Element {
    pub const MASK_LOW_51BITS: u64 = (1 << 51) - 1;
    pub const ZERO: &'static Element = &Element(0, 0, 0, 0, 0);
    pub const ONE: &'static Element = &Element(1, 0, 0, 0, 0);

    // zeroes the given object
    pub fn zero(&mut self) -> &Element {
        self.clone_from(Element::ZERO);
        self
    }
    pub fn one(&mut self) -> &Element {
        self.clone_from(Element::ONE);
        self
    }

    pub fn assign(&mut self, a: &Element) -> &Self {
        self.clone_from(a);
        self
    }

    // assign self to (a + b)
    pub fn add(a: &Element, b: &Element) -> Element {
        let mut v: Element = Element(a.0 + b.0, a.1 + b.1, a.2 + b.2, a.3 + b.3, a.4 + b.4);
        v.carry_propagate();
        v
    }

    // assign self to (a - b)
    pub fn subtract(a: &Element, b: &Element) -> Element {
        let mut v: Element = Element(
            (a.0 + 0xFFFFFFFFFFFDA) - b.0,
            (a.1 + 0xFFFFFFFFFFFFE) - b.1,
            (a.2 + 0xFFFFFFFFFFFFE) - b.2,
            (a.3 + 0xFFFFFFFFFFFFE) - b.3,
            (a.4 + 0xFFFFFFFFFFFFE) - b.4);
        v.carry_propagate();
        v
    }

    // assign self to -a
    pub fn negate(a: &Element) -> Element {
        Element::subtract(Element::ZERO, a)
    }

    pub fn shift_u128_to_u64(a: &Uint128) -> u64 {
        (a.hi << (64 - 51)) | (a.lo >> 51)
    }

    // mul32 sets v = x * y, and returns v.
    pub fn mul32(x: &Element, y: u32) -> Element {
        let (x0lo, x0hi) = Element::mul51(x.0, y);
        let (x1lo, x1hi) = Element::mul51(x.1, y);
        let (x2lo, x2hi) = Element::mul51(x.2, y);
        let (x3lo, x3hi) = Element::mul51(x.3, y);
        let (x4lo, x4hi) = Element::mul51(x.4, y);

        // The hi portions are going to be only 32 bits, plus any previous excess,
        // so we can skip the carry propagation.
        let mut v = Element(
            x0lo + (19 * x4hi), // carried over per the reduction identity
            x1lo + x0hi,
            x2lo + x1hi,
            x3lo + x2hi,
            x4lo + x3hi);
        v.carry_propagate();
        v
    }

    // returns lo + hi * 2⁵¹ = a * b.
    pub fn mul51(a: u64, b: u32) -> (u64, u64) {
        let prod: Uint128 = bits::mul64(a, b as u64);
        let lo = prod.lo & Element::MASK_LOW_51BITS;
        let hi = (prod.hi << 13) | (prod.lo >> 51);
        (lo, hi)
    }

    // calculate x * y.
    pub fn multiply(a: &Element, b: &Element) -> Element {
        let a0 = a.0;
        let a1 = a.1;
        let a2 = a.2;
        let a3 = a.3;
        let a4 = a.4;

        let b0 = b.0;
        let b1 = b.1;
        let b2 = b.2;
        let b3 = b.3;
        let b4 = b.4;

        // Limb multiplication works like pen-and-paper columnar multiplication, but
        // with 51-bit limbs instead of digits.
        //
        //                          a4   a3   a2   a1   a0  x
        //                          b4   b3   b2   b1   b0  =
        //                         ------------------------
        //                        a4b0 a3b0 a2b0 a1b0 a0b0  +
        //                   a4b1 a3b1 a2b1 a1b1 a0b1       +
        //              a4b2 a3b2 a2b2 a1b2 a0b2            +
        //         a4b3 a3b3 a2b3 a1b3 a0b3                 +
        //    a4b4 a3b4 a2b4 a1b4 a0b4                      =
        //   ----------------------------------------------
        //      r8   r7   r6   r5   r4   r3   r2   r1   r0
        //
        // We can then use the reduction identity (a * 2²⁵⁵ + b = a * 19 + b) to
        // reduce the limbs that would overflow 255 bits. r5 * 2²⁵⁵ becomes 19 * r5,
        // r6 * 2³⁰⁶ becomes 19 * r6 * 2⁵¹, etc.
        //
        // Reduction can be carried out simultaneously to multiplication. For
        // example, we do not compute r5: whenever the result of a multiplication
        // belongs to r5, like a1b4, we multiply it by 19 and add the result to r0.
        //
        //            a4b0    a3b0    a2b0    a1b0    a0b0  +
        //            a3b1    a2b1    a1b1    a0b1 19×a4b1  +
        //            a2b2    a1b2    a0b2 19×a4b2 19×a3b2  +
        //            a1b3    a0b3 19×a4b3 19×a3b3 19×a2b3  +
        //            a0b4 19×a4b4 19×a3b4 19×a2b4 19×a1b4  =
        //           --------------------------------------
        //              r4      r3      r2      r1      r0
        //
        // Finally we add up the columns into wide, overlapping limbs.

        let a1_19 = a1 * 19;
        let a2_19 = a2 * 19;
        let a3_19 = a3 * 19;
        let a4_19 = a4 * 19;

        // r0 = a0×b0 + 19×(a1×b4 + a2×b3 + a3×b2 + a4×b1)
        let r0 = bits::mul64(a0, b0);
        let r0 = bits::add_mul64(r0, a1_19, b4);
        let r0 = bits::add_mul64(r0, a2_19, b3);
        let r0 = bits::add_mul64(r0, a3_19, b2);
        let r0 = bits::add_mul64(r0, a4_19, b1);

        // r1 = a0×b1 + a1×b0 + 19×(a2×b4 + a3×b3 + a4×b2)
        let r1 = bits::mul64(a0, b1);
        let r1 = bits::add_mul64(r1, a1, b0);
        let r1 = bits::add_mul64(r1, a2_19, b4);
        let r1 = bits::add_mul64(r1, a3_19, b3);
        let r1 = bits::add_mul64(r1, a4_19, b2);

        // r2 = a0×b2 + a1×b1 + a2×b0 + 19×(a3×b4 + a4×b3)
        let r2 = bits::mul64(a0, b2);
        let r2 = bits::add_mul64(r2, a1, b1);
        let r2 = bits::add_mul64(r2, a2, b0);
        let r2 = bits::add_mul64(r2, a3_19, b4);
        let r2 = bits::add_mul64(r2, a4_19, b3);

        // r3 = a0×b3 + a1×b2 + a2×b1 + a3×b0 + 19×a4×b4
        let r3 = bits::mul64(a0, b3);
        let r3 = bits::add_mul64(r3, a1, b2);
        let r3 = bits::add_mul64(r3, a2, b1);
        let r3 = bits::add_mul64(r3, a3, b0);
        let r3 = bits::add_mul64(r3, a4_19, b4);

        // r4 = a0×b4 + a1×b3 + a2×b2 + a3×b1 + a4×b0
        let r4 = bits::mul64(a0, b4);
        let r4 = bits::add_mul64(r4, a1, b3);
        let r4 = bits::add_mul64(r4, a2, b2);
        let r4 = bits::add_mul64(r4, a3, b1);
        let r4 = bits::add_mul64(r4, a4, b0);

        // After the multiplication, we need to reduce (carry) the five coefficients
        // to obtain a result with limbs that are at most slightly larger than 2⁵¹,
        // to respect the Element invariant.
        //
        // Overall, the reduction works the same as carryPropagate, except with
        // wider inputs: we take the carry for each coefficient by shifting it right
        // by 51, and add it to the limb above it. The top carry is multiplied by 19
        // according to the reduction identity and added to the lowest limb.
        //
        // The largest coefficient (r0) will be at most 111 bits, which guarantees
        // that all carries are at most 111 - 51 = 60 bits, which fits in a uint64.
        //
        //     r0 = a0×b0 + 19×(a1×b4 + a2×b3 + a3×b2 + a4×b1)
        //     r0 < 2⁵²×2⁵² + 19×(2⁵²×2⁵² + 2⁵²×2⁵² + 2⁵²×2⁵² + 2⁵²×2⁵²)
        //     r0 < (1 + 19 × 4) × 2⁵² × 2⁵²
        //     r0 < 2⁷ × 2⁵² × 2⁵²
        //     r0 < 2¹¹¹
        //
        // Moreover, the top coefficient (r4) is at most 107 bits, so c4 is at most
        // 56 bits, and c4 * 19 is at most 61 bits, which again fits in a uint64 and
        // allows us to easily apply the reduction identity.
        //
        //     r4 = a0×b4 + a1×b3 + a2×b2 + a3×b1 + a4×b0
        //     r4 < 5 × 2⁵² × 2⁵²
        //     r4 < 2¹⁰⁷
        //

        let c0: u64 = Element::shift_u128_to_u64(&r0);
        let c1: u64 = Element::shift_u128_to_u64(&r1);
        let c2: u64 = Element::shift_u128_to_u64(&r2);
        let c3: u64 = Element::shift_u128_to_u64(&r3);
        let c4: u64 = Element::shift_u128_to_u64(&r4);

        let rr0 = (r0.lo & Element::MASK_LOW_51BITS) + (c4 * 19);
        let rr1 = (r1.lo & Element::MASK_LOW_51BITS) + c0;
        let rr2 = (r2.lo & Element::MASK_LOW_51BITS) + c1;
        let rr3 = (r3.lo & Element::MASK_LOW_51BITS) + c2;
        let rr4 = (r4.lo & Element::MASK_LOW_51BITS) + c3;

        // Now all coefficients fit into 64-bit registers but are still too large to
        // be passed around as an Element. We therefore do one last carry chain,
        // where the carries will be small enough to fit in the wiggle room above 2⁵¹.
        let mut v: Element = Element(rr0, rr1, rr2, rr3, rr4);
        v.carry_propagate();
        v
    }

    // calculate x * x.
    pub fn square(a: &Element) -> Element {
        let l0 = a.0;
        let l1 = a.1;
        let l2 = a.2;
        let l3 = a.3;
        let l4 = a.4;

        // Squaring works precisely like multiplication above, but thanks to its
        // symmetry we get to group a few terms together.
        //
        //                          l4   l3   l2   l1   l0  x
        //                          l4   l3   l2   l1   l0  =
        //                         ------------------------
        //                        l4l0 l3l0 l2l0 l1l0 l0l0  +
        //                   l4l1 l3l1 l2l1 l1l1 l0l1       +
        //              l4l2 l3l2 l2l2 l1l2 l0l2            +
        //         l4l3 l3l3 l2l3 l1l3 l0l3                 +
        //    l4l4 l3l4 l2l4 l1l4 l0l4                      =
        //   ----------------------------------------------
        //      r8   r7   r6   r5   r4   r3   r2   r1   r0
        //
        //            l4l0    l3l0    l2l0    l1l0    l0l0  +
        //            l3l1    l2l1    l1l1    l0l1 19×l4l1  +
        //            l2l2    l1l2    l0l2 19×l4l2 19×l3l2  +
        //            l1l3    l0l3 19×l4l3 19×l3l3 19×l2l3  +
        //            l0l4 19×l4l4 19×l3l4 19×l2l4 19×l1l4  =
        //           --------------------------------------
        //              r4      r3      r2      r1      r0
        //
        // With precomputed 2×, 19×, and 2×19× terms, we can compute each limb with
        // only three Mul64 and four Add64, instead of five and eight.

        let l0_2 = l0 * 2;
        let l1_2 = l1 * 2;

        let l1_38 = l1 * 38;
        let l2_38 = l2 * 38;
        let l3_38 = l3 * 38;

        let l3_19 = l3 * 19;
        let l4_19 = l4 * 19;

        // r0 = l0×l0 + 19×(l1×l4 + l2×l3 + l3×l2 + l4×l1) = l0×l0 + 19×2×(l1×l4 + l2×l3)
        let r0 = bits::mul64(l0, l0);
        let r0 = bits::add_mul64(r0, l1_38, l4);
        let r0 = bits::add_mul64(r0, l2_38, l3);

        // r1 = l0×l1 + l1×l0 + 19×(l2×l4 + l3×l3 + l4×l2) = 2×l0×l1 + 19×2×l2×l4 + 19×l3×l3
        let r1 = bits::mul64(l0_2, l1);
        let r1 = bits::add_mul64(r1, l2_38, l4);
        let r1 = bits::add_mul64(r1, l3_19, l3);

        // r2 = l0×l2 + l1×l1 + l2×l0 + 19×(l3×l4 + l4×l3) = 2×l0×l2 + l1×l1 + 19×2×l3×l4
        let r2 = bits::mul64(l0_2, l2);
        let r2 = bits::add_mul64(r2, l1, l1);
        let r2 = bits::add_mul64(r2, l3_38, l4);

        // r3 = l0×l3 + l1×l2 + l2×l1 + l3×l0 + 19×l4×l4 = 2×l0×l3 + 2×l1×l2 + 19×l4×l4
        let r3 = bits::mul64(l0_2, l3);
        let r3 = bits::add_mul64(r3, l1_2, l2);
        let r3 = bits::add_mul64(r3, l4_19, l4);

        // r4 = l0×l4 + l1×l3 + l2×l2 + l3×l1 + l4×l0 = 2×l0×l4 + 2×l1×l3 + l2×l2
        let r4 = bits::mul64(l0_2, l4);
        let r4 = bits::add_mul64(r4, l1_2, l3);
        let r4 = bits::add_mul64(r4, l2, l2);

        let c0: u64 = Element::shift_u128_to_u64(&r0);
        let c1: u64 = Element::shift_u128_to_u64(&r1);
        let c2: u64 = Element::shift_u128_to_u64(&r2);
        let c3: u64 = Element::shift_u128_to_u64(&r3);
        let c4: u64 = Element::shift_u128_to_u64(&r4);

        let rr0 = (r0.lo & Element::MASK_LOW_51BITS) + (c4 * 19);
        let rr1 = (r1.lo & Element::MASK_LOW_51BITS) + c0;
        let rr2 = (r2.lo & Element::MASK_LOW_51BITS) + c1;
        let rr3 = (r3.lo & Element::MASK_LOW_51BITS) + c2;
        let rr4 = (r4.lo & Element::MASK_LOW_51BITS) + c3;

        let mut v: Element = Element(rr0, rr1, rr2, rr3, rr4);
        v.carry_propagate();
        v
    }

    // calculate 1/x mod p.
    // If x == 0, returns 0.
    // Inversion is implemented as exponentiation with exponent p − 2. It uses the
    // same sequence of 254 squarings and 11 multiplications as mentioned in [Curve25519].
    pub fn invert(x: &Element) -> Element {
        let x2 = Element::square(x);                    // x^2
        let mut t = Element::square(&x2);               // x^4
        t = Element::square(&t);                        // x^8
        let x9 = Element::multiply(&t, &x);             // x^9
        let x11 = Element::multiply(&x9, &x2);          // x^11
        t = Element::square(&x11);                      // x^22
        let x2_5_0 = Element::multiply(&t, &x9);        // x^31 = x^(2^5 - 2^0)

        t = Element::square(&x2_5_0);                   // x^(2^6 - 2^1)
        // t0 = (2^7 - 2^2); t1 = (2^8 - 2^3); t2 = (2^9 - 2^4); t3 = (2^10 - 2^5)
        for _ in 0..4 {
            t = Element::square(&t);                    // x^(2^10 - 2^5)
        }
        let x2_10_0 = Element::multiply(&t, &x2_5_0);   // x^(2^10 - 2^0) = x^(2^10 - 2^5) * x^(2^5 - 2^0)

        t = Element::square(&x2_10_0);                  // x^(2^11 - 2^1)
        for _ in 0..9 {
            t = Element::square(&t);                    // x^(2^20 - 2^10)
        }
        let x2_20_0 = Element::multiply(&t, &x2_10_0);  // x^(2^20 - 2^0) = x^(2^20 - 2^10) * x^(2^10 - 2^0)

        t = Element::square(&x2_20_0);                  // x^(2^21 - 2^1)
        for _ in 0..19 {
            t = Element::square(&t);                    // x^(2^40 - 2^20)
        }
        t = Element::multiply(&t, &x2_20_0);            // x^(2^40 - 2^0) = x^(2^40 - 2^20) * x^(2^20 - 2^0)

        t = Element::square(&t);                        // x^(2^41 - 2^1)
        for _ in 0..9 {
            t = Element::square(&t);                    // x^(2^50 - 2^10)
        }
        let x2_50_0 = Element::multiply(&t, &x2_10_0);  // x^(2^50 - 2^0) = x^(2^50 - 2^10) * x^(2^10 - 2^0)

        t = Element::square(&x2_50_0);                  // x^(2^51 - 2^1)
        for _ in 0..49 {
            t = Element::square(&t);                    // x^(2^100 - 2^50)
        }
        let x2_100_0 = Element::multiply(&t, &x2_50_0); // x^(2^100 - 2^0) = x^(2^100 - 2^50) * x^(2^50 - 2^0)

        t = Element::square(&x2_100_0);                 // x^(2^101 - 2^1)
        for _ in 0..99 {
            t = Element::square(&t);                    // x^(2^200 - 2^100)
        }
        t = Element::multiply(&t, &x2_100_0);           // x^(2^200 - 2^0) = x^(2^200 - 2^100) * x^(2^100 - 2^0)

        t = Element::square(&t);                        // x^(2^201 - 2^1)
        for _ in 0..49 {
            t = Element::square(&t);                    // x^(2^250 - 2^50)
        }
        t = Element::multiply(&t, &x2_50_0);            // x^(2^250 - 2^0) = x^(2^250 - 2^50) * x^(2^50 - 2^0)

        t = Element::square(&t);                        // x^(2^251 - 2^1)
        t = Element::square(&t);                        // x^(2^252 - 2^2)
        t = Element::square(&t);                        // x^(2^253 - 2^3)
        t = Element::square(&t);                        // x^(2^254 - 2^4)
        t = Element::square(&t);                        // x^(2^255 - 2^5)

        Element::multiply(&t, &x11)                     // x^(2^255 - 21) = x^(2^255 - 2^5) * x^11
    }

    // reduce value modulo 2^255 - 19
    pub fn reduce(&mut self) -> &Self {
        self.carry_propagate();
        // After the light reduction we now have a field element representation
        // v < 2^255 + 2^13 * 19, but need v < 2^255 - 19.

        // If v >= 2^255 - 19, then v + 19 >= 2^255, which would overflow 2^255 - 1,
        // generating a carry. That is, c will be 0 if v < 2^255 - 19, and 1 otherwise.
        let mut c = (self.0 + 19) >> 51;
        c = (self.1 + c) >> 51;
        c = (self.2 + c) >> 51;
        c = (self.3 + c) >> 51;
        c = (self.4 + c) >> 51;

        // If v < 2^255 - 19 and c = 0, this will be a no-op. Otherwise, it's
        // effectively applying the reduction identity to the carry.
        self.0 += 19 * c;

        self.1 += self.0 >> 51;
        self.0 = self.0 & Element::MASK_LOW_51BITS;
        self.2 += self.1 >> 51;
        self.1 = self.1 & Element::MASK_LOW_51BITS;
        self.3 += self.2 >> 51;
        self.2 = self.2 & Element::MASK_LOW_51BITS;
        self.4 += self.3 >> 51;
        self.3 = self.3 & Element::MASK_LOW_51BITS;
        // no additional carry
        self.4 = self.4 & Element::MASK_LOW_51BITS;

        self
    }

    pub fn carry_propagate(&mut self) {
        let c0 = self.0 >> 51;
        let c1 = self.1 >> 51;
        let c2 = self.2 >> 51;
        let c3 = self.3 >> 51;
        let c4 = self.4 >> 51;

        self.0 = (self.0 & Element::MASK_LOW_51BITS) + (c4 * 19);
        self.1 = (self.1 & Element::MASK_LOW_51BITS) + c0;
        self.2 = (self.2 & Element::MASK_LOW_51BITS) + c1;
        self.3 = (self.3 & Element::MASK_LOW_51BITS) + c2;
        self.4 = (self.4 & Element::MASK_LOW_51BITS) + c3;
    }

    pub fn to_le_bytes(&self) -> [u8; 32] {
        let mut b: [u8; 32] = [0; 32];
        let mut el = self.clone();
        el.le_bytes(&mut b)
    }

    pub fn le_bytes(&mut self, b: &mut [u8; 32]) -> [u8; 32] {
        self.reduce(); // applies mask 51 for all limbs
        b.fill(0);
        // Bits 0:51 (bytes 0:8, bits 0:64, shift 0, mask 51).
        binary::LittleEndian::put_u64(&mut b[0..8], self.0);
        // Bits 51:102 (bytes 6:14, bits 48:112, shift 3, mask 51).
        let val: u64 = (self.1 << 3) | (b[6] as u64);
        binary::LittleEndian::put_u64(&mut b[6..14], val);
        // Bits 102:153 (bytes 12:20, bits 96:160, shift 6, mask 51).
        let val: u64 = (self.2 << 6) | (b[12] as u64); // 63
        binary::LittleEndian::put_u64(&mut b[12..20], val);
        // Bits 153:204 (bytes 19:27, bits 152:216, shift 1).
        let val: u64 = (self.3 << 1) | (b[19] as u64);
        binary::LittleEndian::put_u64(&mut b[19..27], val);
        // Bits 204:255 (bytes 24:32, bits 192:256, shift 12, mask 51).
        // Note: not bytes 25:33, shift 12, to avoid over-read.
        let val: u64 = (self.4 << 12) | (((b[25] as u64) << 8) | b[24] as u64);
        binary::LittleEndian::put_u64(&mut b[24..32], val);

        *b
    }

    // from_le_bytes initializes the five limbs from an array of 32-octets stored in little-endian encoding.
    // Consistent with RFC 7748, the most significant bit (the high bit of the last byte)
    // is ignored, and non-canonical values (2^255-19 through 2^255-1) are accepted.
    // This is laxer than specified by RFC 8032, but consistent with most Ed25519 implementations.
    pub fn from_le_bytes(b: [u8; 32]) -> Self {
        let mut el = Element::ZERO.clone();
        Element::init_from_le_bytes(&mut el, b);
        el
    }

    pub fn init_from_le_bytes(&mut self, b: [u8; 32]) {
        // Bits 0:51 (bytes 0:8, bits 0:64, shift 0, mask 51).
        let l: [u8; 8] = b[0..8].try_into().unwrap();
        self.0 = binary::LittleEndian::u64(l) & Element::MASK_LOW_51BITS;
        // Bits 51:102 (bytes 6:14, bits 48:112, shift 3, mask 51).
        let l: [u8; 8] = b[6..14].try_into().unwrap();
        self.1 = (binary::LittleEndian::u64(l) >> 3) & Element::MASK_LOW_51BITS;
        // Bits 102:153 (bytes 12:20, bits 96:160, shift 6, mask 51).
        let l: [u8; 8] = b[12..20].try_into().unwrap();
        self.2 = (binary::LittleEndian::u64(l) >> 6) & Element::MASK_LOW_51BITS;
        // Bits 153:204 (bytes 19:27, bits 152:216, shift 1, mask 51).
        let l: [u8; 8] = b[19..27].try_into().unwrap();
        self.3 = (binary::LittleEndian::u64(l) >> 1) & Element::MASK_LOW_51BITS;
        // Bits 204:255 (bytes 24:32, bits 192:256, shift 12, mask 51).
        // Note: not bytes 25:33, shift 4, to avoid over-read.
        let l: [u8; 8] = b[24..32].try_into().unwrap();
        self.4 = (binary::LittleEndian::u64(l) >> 12) & Element::MASK_LOW_51BITS;
    }

    pub fn from_bytes(b: [u8; 32]) -> Self {
        let mut el = Self::ZERO.clone();
        el.init_from_le_bytes(b);
        el
    }

    // returns 1 if v and u are equal, and 0 otherwise.
    pub fn equal(&self, u: &Element) -> bool {
        let (sa, sv) = (u.to_le_bytes(), self.to_le_bytes());
        subtle::constant_time_compare(&sa, &sv)
    }

    // returns 0xFFFFFFFFFFFFFFFF if cond is 1, and 0 otherwise.
    pub fn mask_64bits(cond: u32) -> u64 {
        if cond == 1 {
            0xFFFFFFFFFFFFFFFF
        } else {
            0
        }
    }

    // Select sets v to a if cond == 1, and to b if cond == 0.
    pub fn select(&mut self, a: &Element, b: &Element, cond: u32) {
        let m = Element::mask_64bits(cond);
        self.0 = (m & a.0) | (!m & b.0);
        self.1 = (m & a.1) | (!m & b.1);
        self.2 = (m & a.2) | (!m & b.2);
        self.3 = (m & a.3) | (!m & b.3);
        self.4 = (m & a.4) | (!m & b.4);
    }

    pub fn swap(s: &mut Element, u: &mut Element, cond: u32) {
        let m: u64 = Element::mask_64bits(cond);
        let t = m & (s.0 ^ u.0);
        s.0 ^= t;
        u.0 ^= t;
        let t = m & (s.1 ^ u.1);
        s.1 ^= t;
        u.1 ^= t;
        let t = m & (s.2 ^ u.2);
        s.2 ^= t;
        u.2 ^= t;
        let t = m & (s.3 ^ u.3);
        s.3 ^= t;
        u.3 ^= t;
        let t = m & (s.4 ^ u.4);
        s.4 ^= t;
        u.4 ^= t;
    }
}

#[cfg(test)]
mod field_test {
    use crate::field;
    use crate::field::Element;

    #[test]
    fn test_mask_low_51bits() {
        assert_eq!(u64::MAX & Element::MASK_LOW_51BITS, 0x7ffffffffffff);
        assert_eq!(u64::MAX & Element::MASK_LOW_51BITS, Element::MASK_LOW_51BITS);
        assert_eq!(Element::MASK_LOW_51BITS + 1 & Element::MASK_LOW_51BITS, 0);
    }

    #[test]
    fn test_elem_zero_eq() {
        assert_eq!(Element::ZERO, Element::ZERO);
        assert_ne!(Element::ZERO, Element::ONE);
        let z: &mut Element = &mut Element(u64::MAX, u64::MAX, u64::MAX, u64::MAX, 1);
        assert_ne!(z, Element::ZERO);
        assert_ne!(z, Element::ONE);
        assert_eq!(z.0 & z.1, u64::MAX);
        assert_eq!(z.0 & Element::MASK_LOW_51BITS, Element::MASK_LOW_51BITS);

        let _ = z.zero();
        assert_eq!(z, Element::ZERO);
        assert_eq!(z.0 & z.1, 0);
    }

    #[test]
    fn test_elem_one_eq() {
        assert_eq!(Element::ONE, Element::ONE);
        assert_ne!(Element::ZERO, Element::ONE);
        let d: &mut Element = &mut Element(u64::MAX, u64::MAX, u64::MAX, u64::MAX, 0);
        assert_ne!(d, Element::ZERO);
        assert_ne!(d, Element::ONE);
        assert_eq!(d.0 & d.1, u64::MAX);
        assert_eq!(d.4, 0);
        assert_eq!(d.0 & Element::MASK_LOW_51BITS, Element::MASK_LOW_51BITS);

        let _ = d.one();
        assert_eq!(d, Element::ONE);
        assert_eq!(d.0, 1);
        assert_eq!(d.4, 0);
        assert_eq!(d.0 & Element::MASK_LOW_51BITS, 1);
    }

    #[test]
    fn test_set_limbs_01() {
        let bytes: [u8; 32] = [74, 209, 69, 197, 70, 70, 161, 222, 56, 226, 229, 19, 112, 60, 25, 92, 187, 74, 222, 56, 50, 153, 51, 233, 40, 74, 57, 6, 160, 185, 213, 31];
        let expect = Element(358744748052810, 1691584618240980, 977650209285361, 1429865912637724, 560044844278676);
        let d: &mut Element = &mut Element::ZERO.clone();
        let _ = d.init_from_le_bytes(bytes);
        assert_eq!(d.clone(), expect);
    }

    #[test]
    fn test_set_limbs_02() {
        let bytes: [u8; 32] = [199, 23, 106, 112, 61, 77, 216, 79, 186, 60, 11, 118, 13, 16, 103, 15, 42, 32, 83, 250, 44, 57, 204, 198, 78, 199, 253, 119, 146, 172, 3, 122];
        let expect = Element(84926274344903, 473620666599931, 365590438845504, 1028470286882429, 2146499180330972);
        let d: &mut Element = &mut Element::ZERO.clone();
        let _ = d.init_from_le_bytes(bytes);
        assert_eq!(d.clone(), expect);
    }

    #[test]
    fn test_set_limbs_from_bytes() {
        let bytes: [u8; 32] = [199, 23, 106, 112, 61, 77, 216, 79, 186, 60, 11, 118, 13, 16, 103, 15, 42, 32, 83, 250, 44, 57, 204, 198, 78, 199, 253, 119, 146, 172, 3, 122];
        let expect = Element(84926274344903, 473620666599931, 365590438845504, 1028470286882429, 2146499180330972);
        let d: Element = Element::from_bytes(bytes);
        assert_eq!(d.clone(), expect);
    }

    #[test]
    fn test_to_bytes_01() {
        let expect: [u8; 32] = [74, 209, 69, 197, 70, 70, 161, 222, 56, 226, 229, 19, 112, 60, 25, 92, 187, 74, 222, 56, 50, 153, 51, 233, 40, 74, 57, 6, 160, 185, 213, 31];
        let el = Element(358744748052810, 1691584618240980, 977650209285361, 1429865912637724, 560044844278676);
        let bytes = el.to_le_bytes();
        assert_eq!(expect, bytes);
    }

    #[test]
    fn test_to_bytes_02() {
        let expect: [u8; 32] = [199, 23, 106, 112, 61, 77, 216, 79, 186, 60, 11, 118, 13, 16, 103, 15, 42, 32, 83, 250, 44, 57, 204, 198, 78, 199, 253, 119, 146, 172, 3, 122];
        let el = Element(84926274344903, 473620666599931, 365590438845504, 1028470286882429, 2146499180330972);
        let bytes = el.to_le_bytes();
        assert_eq!(expect, bytes);
    }

    #[test]
    fn test_swap_01() {
        let mut a = Element(358744748052810, 1691584618240980, 977650209285361, 1429865912637724, 560044844278676);
        let mut b = Element(84926274344903, 473620666599931, 365590438845504, 1028470286882429, 2146499180330972);

        let mut c = Element::ZERO.clone();
        let mut d = Element::ZERO.clone();

        c.select(&mut a, &mut b, 1);
        d.select(&mut a, &mut b, 0);

        assert!(c.equal(&a) && d.equal(&b));
        Element::swap(&mut c, &mut d, 0);
        assert!(c.equal(&a) && d.equal(&b));
        Element::swap(&mut c, &mut d, 1);
        assert!(c.equal(&b) && d.equal(&a));
    }

    #[test]
    fn test_25524_div_8() {

        // 2^255 - 24
        let bytes_25524: [u8; 32] = [
            0xe8, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f,
        ];
        let fe_25524 = field::Element::from_le_bytes(bytes_25524);
        // // (2^255 - 19) - 5 == (2^255 - 24)
        {
            // 2^255 - 19
            let bytes_25519: [u8; 32] = [
                0xed, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f,
            ];
            let fe_25519 = field::Element::from_le_bytes(bytes_25519);
            let five: [u8; 32] = [
                5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ];
            assert_eq!(fe_25524,
                       //(2^255 - 19) - 5
                       field::Element::subtract(&fe_25519,
                                                &field::Element::from_le_bytes(five)));
        }
        // (2^255 - 24)/8 == (2^252 - 3)
        {
            let bytes_25203: [u8; 32] = [
                0xfd, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0f,
            ];
            let fe_25203 = field::Element::from_le_bytes(bytes_25203);

            let eight: [u8; 32] = [
                8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ];
            let fe_8 = field::Element::from_le_bytes(eight);
            let mut inv_8 = field::Element::invert(&fe_8);
            inv_8.reduce();

            let mut r = field::Element::multiply(&fe_25524, &inv_8);
            r.reduce();
            assert_eq!(r, fe_25203);
        }

        // 2^252 - 24
        {
            let bytes_25224: [u8; 32] = [
                0xe8, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0f,
            ];
            let pow_2_252: [u8; 32] = [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0x10,
            ];
            let twenty_four: [u8; 32] = [
                24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ];
            let fe_2_252 = field::Element::from_le_bytes(pow_2_252);
            let fe_24 = field::Element::from_bytes(twenty_four);
            let mut fe_252_24 = field::Element::subtract(&fe_2_252, &fe_24);
            fe_252_24.reduce();
            assert_eq!(fe_252_24, field::Element::from_le_bytes(bytes_25224))
        }
    }

    #[test]
    fn test_calc_rfc8032_d() {
        // d of edwards25519 in [RFC7748] = -121665/121666
        // d = 37095705934669439343138083508754565189542113879843219016388785533085940283555
        //   = 0x52036cee2b6ffe738cc740797779e89800700a4d4141d8ab75eb4dca135978a3
        let bytes_d: [u8; 32] = [
            0xa3, 0x78, 0x59, 0x13, 0xca, 0x4d, 0xeb, 0x75, 0xab, 0xd8, 0x41,
            0x41, 0x4d, 0x0a, 0x70, 0x00, 0x98, 0xe8, 0x79, 0x77, 0x79, 0x40,
            0xc7, 0x8c, 0x73, 0xfe, 0x6f, 0x2b, 0xee, 0x6c, 0x03, 0x52,
        ];
        let fe_rfc7748_d = field::Element::from_le_bytes(bytes_d);

        let bytes_121665: [u8; 32] = [
            0x41, 0xdb, 0x01,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let fe_121665 = field::Element::from_le_bytes(bytes_121665);

        let bytes_121666: [u8; 32] = [
            0x42, 0xdb, 0x01,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let fe_121666 = field::Element::from_le_bytes(bytes_121666);

        let inv_121666 = field::Element::invert(&fe_121666);
        let d = field::Element::multiply(&fe_121665, &inv_121666);
        let neg_d = field::Element::negate(&d);

        assert_eq!(neg_d, fe_rfc7748_d);
    }
}
