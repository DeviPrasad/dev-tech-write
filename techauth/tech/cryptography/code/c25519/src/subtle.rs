// constant_time_compare returns true if the two slices, x and y, have equal contents
// and false otherwise. The time taken is a function of the length of the slices and
// is independent of the contents. If the lengths of x and y do not match it
// returns false immediately.
pub fn constant_time_compare(x: &[u8], y: &[u8]) -> bool {
    if x.len() != y.len() {
        false
    } else {
        let mut v: u8 = 0;
        for i in 0..x.len() {
            v |= x[i] ^ y[i];
        }
        v == 0
    }
}

// isZero returns whether a is all zeroes in constant time.
pub fn is_zero(a: &[u8]) -> bool {
    let mut acc: u8 = 0;
    for b in a {
        acc |= b
    }
    acc == 0
}
