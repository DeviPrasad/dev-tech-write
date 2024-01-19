#![allow(dead_code)]

use crate::binary;

// Element represents an element of the field GF(2^255-19).
// An element is represented as a radix-2^51 value.
// An element t represents the integer
//     t.l0 + t.1*2^51 + t.l2*2^102 + t.l3*2^153 + t.l4*2^204
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
    pub fn add(&mut self, a: &Element, b: &Element) -> &Self {
        self.0 = a.0 + b.0;
        self.1 = a.1 + b.1;
        self.2 = a.2 + b.2;
        self.3 = a.3 + b.3;
        self.4 = a.4 + b.4;

        self.carry_propagate()
    }

    // assign self to (a - b)
    pub fn subtract(&mut self, a: &Element, b: &Element) -> &Self {
        self.0 = (a.0 + 0xFFFFFFFFFFFDA) - b.0;
        self.1 = (a.1 + 0xFFFFFFFFFFFFE) - b.1;
        self.2 = (a.2 + 0xFFFFFFFFFFFFE) - b.2;
        self.3 = (a.3 + 0xFFFFFFFFFFFFE) - b.3;
        self.4 = (a.4 + 0xFFFFFFFFFFFFE) - b.4;

        self.carry_propagate()
    }

    // assign self to -a
    pub fn negate(&mut self, a: &Element) -> &Self {
        self.subtract(Element::ZERO, a)
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

    pub fn carry_propagate(&mut self) -> &Self {
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

        self
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
}

#[cfg(test)]
mod field_test {
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
}