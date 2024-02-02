#![allow(dead_code)]

#[derive(Debug, Eq, PartialEq)]
pub struct Uint128 {
    pub(crate) lo: u64,
    pub(crate) hi: u64
}

// _generic_mul64_ returns the 128-bit product of x and y: (hi, lo) = x * y.
// The product's upper half is returned in hi and the lower half in lo.
// This is equivalent to the implementation in golang (math/bits/bits.go::Mul64).
// 'wrapping_mul' prevents overflow error (runtime).
// This function's execution time does not depend on the inputs.
pub fn _go_math_mul64_(x: u64, y: u64) -> Uint128 {
    const MASK32: u64 = (1 << 32) - 1;
    let xl: u64 = x & MASK32;
    let xh: u64 = x >> 32;
    let yl: u64 = y  & MASK32;
    let yh: u64 = y >> 32;
    let w0: u64 = xl * yl;
    let ll_carry: u64 = w0 >> 32;
    let hl_sum: u64 = (xh * yl) + ll_carry;
    let hl_mid: u64 = hl_sum & MASK32;
    let hl_carry: u64 = hl_sum >> 32;
    let sum_mid: u64 = (xl * yh) + hl_mid;
    let sum_mid_carry: u64 = sum_mid >> 32;
    let hi: u64 = (xh * yh) + hl_carry + sum_mid_carry;
    let lo: u64 = x.wrapping_mul(y); // Wrapping (modular) multiplication.
    Uint128 { lo, hi }
}

// generic_mul64 is arguably a more readable version of _generic_mul64_.
pub fn _mul64_(x: u64, y: u64) -> Uint128 {
    const MASK32: u64 = (1 << 32) - 1;
    let xl: u64 = x & MASK32;
    let xh: u64 = x >> 32;
    let yl: u64 = y  & MASK32;
    let yh: u64 = y >> 32;

    let ll_prod: u64 = xl * yl;
    let lh_prod: u64 = xl * yh;
    let hl_prod: u64 = xh * yl;
    let hh_prod: u64 = xh * yh;

    let ll_carry: u64 = ll_prod >> 32;
    let carry: u64 = ((lh_prod & MASK32) + (hl_prod & MASK32) + ll_carry) >> 32;

    let hi: u64 = hh_prod + (lh_prod >> 32) + (hl_prod >> 32) + carry;
    let lo: u64 = x.wrapping_mul(y); // Wrapping (modular) multiplication.

    Uint128 { lo, hi }
}

// This is the simplest version of mul64.
// We make use of Rust's support for 128-bit arithmetic.
pub fn rust_mul64(x: u64, y: u64) -> Uint128 {
    let r2: u128 = x as u128 * y as u128;
    Uint128 { lo: r2 as u64, hi: (r2 >> 64) as u64 }
}

pub fn mul64(x: u64, y: u64) -> Uint128 {
    let r = rust_mul64(x, y);
    assert_eq!(r, _mul64_(x, y));
    assert_eq!(r, _go_math_mul64_(x, y));
    r
}

// The carry input must be 0 or 1; otherwise the behavior is undefined.
// carry_out is guaranteed to be 0 or 1.
// This function's execution time does not depend on the inputs.
pub fn add64(x: u64, y: u64, carry: u64) -> (u64, u64) {
    assert!(carry <= 1);
    //
    // This golang implementation is replaced using Rust's 128-bit arithmetic.
    // let sum: u64 = x + y + carry;
    // The sum will overflow if both top bits are set (x & y) or if one of them
    // is (x | y), and a carry from the lower place happened. If such a carry
    // happens, the top bit of sum will be zero (1 + 0 + 1 = 0).
    // let carry_out: u64 = ((x & y) | ((x | y) & !sum)) >> 63;
    //
    let sum: u128 = x as u128 + y as u128 + carry as u128;
    let carry_out: u64 = (sum >> 64) as u64;
    assert!(carry_out <= 1);
    (sum as u64, carry_out)
}

// addMul64 returns v + x * y.
pub fn add_mul64(v: Uint128, x: u64, y: u64) -> Uint128 {
    let r: Uint128 = mul64(x, y);
    let (lo, c) = add64(r.lo, v.lo, 0);
    let (hi, _) = add64(r.hi, v.hi, c);
    Uint128 { lo, hi }
}


#[cfg(test)]
mod bit_tests {
    use crate::bits;
    use crate::bits::Uint128;

    #[test]
    fn test_u64_mul_01() {
        let x: u64 = 0x00000000FFFFFFFF;
        let y: u64 = 0x00000000FFFFFFFF;
        let res: u64 = x * y;
        assert_eq!(res, 0xFFFFFFFE00000001);

        let r: Uint128 = bits::mul64(x, y);
        assert_eq!(r.lo, 0xFFFFFFFE00000001);
        assert_eq!(r.hi, 0);
    }

    #[test]
    fn test_u64_mul_02() {
        let x: u64 = 0x00000001FFFFFFFF;
        let y: u64 = 0x00000001FFFFFFFF;
        // result = 0x3FFFFFFFC00000001
        let (res, overflow): (u64, bool) = x.overflowing_mul(y);
        assert!(overflow);
        assert_eq!(res, 0xFFFFFFFC00000001);

        let r: Uint128 = bits::mul64(x, y);
        assert_eq!(r.lo, 0xFFFFFFFC00000001);
        assert_eq!(r.hi, 0x0000000000000003);
    }

    #[test]
    fn test_u64_mul_57bit_val() {
        let x: u64 = 0x01FFFFFFFFFFFFFF;
        let y: u64 = 0x01FFFFFFFFFFFFFF;
        // result = 0x0003FFFFFFFFFFFF_FC00000000000001
        let r: Uint128 = bits::mul64(x, y);
        assert_eq!(r.lo, 0xFC00000000000001);
        assert_eq!(r.hi, 0x0003FFFFFFFFFFFF);

        let z: u64 = 0;
        let r: Uint128 = bits::mul64(x, z);
        assert_eq!(r.lo, 0);
        assert_eq!(r.hi, 0);
    }

    #[test]
    fn test_u64_mul_03() {
        let x: u64 = 0xFFFFFFFFFFFFFFFF;
        let y: u64 = 0x01FFFFFFFFFFFFFF;
        // result = 0x1FFFFFFFFFFFFFE_FE00000000000001
        let r: Uint128 = bits::mul64(x, y);
        assert_eq!(r.lo, 0xFE00000000000001);
        assert_eq!(r.hi, 0x01FFFFFFFFFFFFFE);

        let r: Uint128 = bits::mul64(x, 1);
        //eprintln!("{hi:X} {lo:X} ");
        assert_eq!(r.lo, 0xFFFFFFFFFFFFFFFF);
        assert_eq!(r.hi, 0);

        let r: Uint128 = bits::mul64(x, 2);
        assert_eq!(r.lo, 0xFFFFFFFFFFFFFFFE);
        assert_eq!(r.hi, 1);
    }

    #[test]
    fn test_u64_mul_04() {
        let x: u64 = 0xFFFFFFFFFFFFFFFF;
        let y: u64 = 0xE1FFFFFFFFFFFFFF;
        // result = 0xE1FFFFFFFFFFFFFE_1E00000000000001
        let r: Uint128 = bits::mul64(x, y);
        assert_eq!(r.lo, 0x1E00000000000001);
        assert_eq!(r.hi, 0xE1FFFFFFFFFFFFFE);
    }

    #[test]
    fn test_u64_mul_05() {
        let x: u64 = 0xFFFFFFFFFFFFFFFF;
        let y: u64 = 0xFFFFFFFFFFFFFFFF;
        // result = 0xFFFFFFFFFFFFFFFE_0000000000000001
        let r: Uint128 = bits::mul64(x, y);
        assert_eq!(r.lo, 0x0000000000000001);
        assert_eq!(r.hi, 0xFFFFFFFFFFFFFFFE);

        let r2: u128 = x as u128 * y as u128;
        let (hi, lo): (u64, u64) = ((r2 >> 64) as u64, r2 as u64);
        assert_eq!(r2, 0xFFFFFFFFFFFFFFFE_0000000000000001);
        assert_eq!(lo, 0x0000000000000001);
        assert_eq!(hi, 0xFFFFFFFFFFFFFFFE);
    }
    #[test]
    // from golang's fe_test.go - TestMul64to128
    fn test_mul_64_to_128() {
        let x: u64 = 18014398509481983; // 2^54 - 1
        let y: u64 = 18014398509481983; // 2^54 - 1
        // result = 0x00000FFFFFFFFFFF_FF80000000000001
        let r: Uint128 = bits::mul64(x, y);
        assert_eq!(r.lo, 0xFF80000000000001);
        assert_eq!(r.hi, 0x00000FFFFFFFFFFF);

        let x: u64 = 1125899906842661;
        let y: u64 = 2097155; // 2^54 - 1
        // result = 0x00000FFFFFFFFFFF_FF80000000000001
        let r: Uint128 = bits::mul64(x, y);
        let r: Uint128 = bits::add_mul64(r, x, y);
        let r: Uint128 = bits::add_mul64(r, x, y);
        let r: Uint128 = bits::add_mul64(r, x, y);
        let r: Uint128 = bits::add_mul64(r, x, y);
        assert_eq!(r.lo, 16888498990613035);
        assert_eq!(r.hi, 640);
    }
}
