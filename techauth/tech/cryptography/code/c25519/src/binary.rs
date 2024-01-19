#![allow(dead_code)]

pub struct LittleEndian {}

impl LittleEndian {
    // mimic golang implementation from encoding/binary/binary.go
    // LittleEndian.PutUint64
    pub fn put_u64(b: &mut [u8], v: u64) {
        b[0] = v as u8;
        b[1] = (v >> 8) as u8;
        b[2] = (v >> 16) as u8;
        b[3] = (v >> 24) as u8;
        b[4] = (v >> 32) as u8;
        b[5] = (v >> 40) as u8;
        b[6] = (v >> 48) as u8;
        b[7] = (v >> 56) as u8;
    }

    // LittleEndian.Uint64
    pub fn get_u64(b: [u8; 8]) -> u64 {
        b[0] as u64 |
            ((b[1] as u64) << 8) |
            ((b[2] as u64) << 16) |
            ((b[3] as u64) << 24) |
            ((b[4] as u64) << 32) |
            ((b[5] as u64) << 40) |
            ((b[6] as u64) << 48) |
            ((b[7] as u64) << 56)
    }

    // straightforward implementation in Rust
    pub fn u64_to_bytes(b: &mut [u8; 8], v: u64) {
        b.copy_from_slice(&v.to_le_bytes());
    }
    pub fn u64(b: [u8; 8]) -> u64 {
        u64::from_le_bytes(b)
    }
}

#[cfg(test)]
mod bin_tests {
    use crate::binary::LittleEndian;

    #[test]
    fn test_u64_le_bytes_01() {
        // let bytes = 0x1234567890123456u64.to_le_bytes();
        let mut b1: [u8; 8] = [0; 8];
        let mut b2: [u8; 8] = [0; 8];
        LittleEndian::put_u64(&mut b1, 0x1234567890123456u64);
        LittleEndian::u64_to_bytes(&mut b2, 0x1234567890123456u64);
        assert_eq!(b1, b2);

        let v1 = LittleEndian::u64(b1);
        let v2 = LittleEndian::get_u64(b1);
        assert_eq!(v1, v2);

        let v1 = LittleEndian::u64(b1);
        let v2 = LittleEndian::get_u64(b2);
        assert_eq!(v1, v2);

        let v1 = LittleEndian::u64(b2);
        let v2 = LittleEndian::get_u64(b1);
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_u64_le_bytes_02() {
        let mut b1: [u8; 8] = [0; 8];
        let mut b2: [u8; 8] = [0; 8];
        LittleEndian::put_u64(&mut b1, u64::MAX);
        LittleEndian::u64_to_bytes(&mut b2, u64::MAX);
        assert_eq!(b1, b2);
    }

    #[test]
    fn test_u64_le_val_01() {
        let mut b1: [u8; 8] = [0; 8];
        let mut b2: [u8; 8] = [0; 8];
        LittleEndian::put_u64(&mut b1, u64::MAX);
        LittleEndian::u64_to_bytes(&mut b2, u64::MAX);
        assert_eq!(b1, b2);

        let v1 = LittleEndian::u64(b1);
        let v2 = LittleEndian::get_u64(b2);
        assert_eq!(v1, u64::MAX);
        assert_eq!(v2, u64::MAX);
    }

    #[test]
    fn test_u64_le_val_02() {
        let mut b1: [u8; 8] = [0; 8];
        let mut b2: [u8; 8] = [0; 8];
        LittleEndian::put_u64(&mut b1, 100200300400);
        LittleEndian::u64_to_bytes(&mut b2, u64::MAX);
        assert_ne!(b1, b2);

        let v1 = LittleEndian::u64(b1);
        let v2 = LittleEndian::get_u64(b2);
        assert_eq!(v1, 100200300400);
        assert_eq!(v2, u64::MAX);

        let v1 = LittleEndian::get_u64(b1);
        let v2 = LittleEndian::u64(b2);
        assert_eq!(v1, 100200300400);
        assert_eq!(v2, u64::MAX);
    }
}
