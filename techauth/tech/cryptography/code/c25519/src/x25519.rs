use crate::field;
use crate::subtle;

// scalar: little endian sequence of bytes
// point: little endian sequence of bytes
pub fn scalar_mul(scalar: [u8; 32], point: [u8; 32], dst: &mut [u8; 32]) {
    fn scalar_clamp(scalar: [u8; 32]) -> [u8; 32] {
        let mut clamped: [u8; 32] = scalar.clone();
        clamped[0] &= 0xF8;
        clamped[31] = (clamped[31] & 0x7F) | 0x40;
        clamped
    }

    let clamped = scalar_clamp(scalar);
    let x1 = field::Element::from_le_bytes(point);
    let mut x2 = field::Element::ONE.clone();
    let mut x3 = x1.clone();
    let mut z2 = field::Element::ZERO.clone();
    let mut z3 = field::Element::ONE.clone();
    let mut swap: u32 = 0;

    for pos in (0..=254).rev() {
        let bit: u32 = ((clamped[pos / 8] >> (pos & 7)) & 1) as u32;
        swap ^= bit;
        field::Element::swap(&mut x2, &mut x3, swap);
        field::Element::swap(&mut z2, &mut z3, swap);
        swap = bit;

        let t0 = field::Element::subtract(&x3, &z3);
        let t1 = field::Element::subtract(&x2, &z2);
        x2 = field::Element::add(&x2, &z2);
        z2 = field::Element::add(&x3, &z3);
        z3 = field::Element::multiply(&t0, &x2);
        z2 = field::Element::multiply(&z2, &t1);
        let t0 = field::Element::square(&t1);
        let t1 = field::Element::square(&x2);
        x3 = field::Element::add(&z3, &z2);
        z2 = field::Element::subtract(&z3, &z2);
        x2 = field::Element::multiply(&t1, &t0);
        let t1 = field::Element::subtract(&t1, &t0);
        z2 = field::Element::square(&z2);
        z3 = field::Element::mul32(&t1, 121666);
        x3 = field::Element::square(&x3);
        let t0 = field::Element::add(&t0, &z3);
        z3 = field::Element::multiply(&x1, &z2);
        z2 = field::Element::multiply(&t1, &t0)
    }

    field::Element::swap(&mut x2, &mut x3, swap);
    field::Element::swap(&mut z2, &mut z3, swap);

    z2 = field::Element::invert(&z2);
    x2 = field::Element::multiply(&x2, &z2);

    x2.le_bytes(dst);
}

pub struct PrivateKey {
    private: [u8; 32],
}

#[derive(Debug)]
pub struct PublicKey {
    public: [u8; 32],
}

impl PrivateKey {
    pub fn new(key: [u8; 32]) -> PrivateKey {
        assert!(!subtle::is_zero(&key));
        PrivateKey {
            private: key
        }
    }

    pub fn public_key(&self) -> PublicKey {
        let x25519_base_point: [u8; 32] = [
            9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(x25519_base_point[0], 9);
        let mut pub_key_bytes: [u8; 32] = [0; 32];
        assert!(!subtle::is_zero(&x25519_base_point));
        scalar_mul(self.private, x25519_base_point, &mut pub_key_bytes);
        assert!(!subtle::is_zero(&pub_key_bytes));
        PublicKey::new(pub_key_bytes)
    }

    pub fn ecdh(&self, peer: &PublicKey) -> Result<[u8; 32], ()> {
        let mut ss: [u8; 32] = [0; 32];
        assert!(!subtle::is_zero(&self.private));
        scalar_mul(self.private, peer.public, &mut ss);
        if subtle::is_zero(&ss) {
            Err(())
        } else {
            Ok(ss)
        }
    }
}

impl PublicKey {
    pub fn new(key: [u8; 32]) -> PublicKey {
        PublicKey {
            public: key
        }
    }
}

#[cfg(test)]
mod test_x25519 {
    use rand::Rng;

    use crate::field;
    use crate::x25519::{PrivateKey, PublicKey};

    fn hex_decode(s: &str) -> Vec<u8> {
        let r = hex::decode(s);
        assert!(r.is_ok());
        r.unwrap()
    }

    #[test]
    fn test_failure_01() {
        fn _test_x25519_fail_(private: [u8; 32], public: [u8; 32]) {
            let pr_key = PrivateKey::new(private);
            let pub_key = PublicKey::new(public);
            let res = pr_key.ecdh(&pub_key);
            assert!(matches!(res, Err(())));
        }

        let identity: &[u8] = &hex_decode("0000000000000000000000000000000000000000000000000000000000000000");
        let low_order_point: &[u8] = &hex_decode("e0eb7a7c3b41b8ae1656e3faf19fc46ada098deb9c32b1fd866205165f49b800");

        let mut random_scalar: [u8; 32] = [0; 32];
        rand::thread_rng().fill(&mut random_scalar);

        _test_x25519_fail_(random_scalar, identity.try_into().unwrap());
        _test_x25519_fail_(random_scalar, low_order_point.try_into().unwrap());
    }

    // https://github.com/AdoptOpenJDK/openjdk-jdk/blob/master/test/jdk/sun/security/ec/xec/TestXDH.java
    #[test]
    fn test_failure_02() {
        fn _test_x25519_failure_(pr_key_hex: &str, peer_pub_key_hex: &str) {
            let pr_key_bytes: [u8; 32] = hex_decode(pr_key_hex).try_into().unwrap();
            let pr_key = PrivateKey::new(pr_key_bytes);
            let peer_pub_key_bytes: [u8; 32] = hex_decode(peer_pub_key_hex).try_into().unwrap();
            let peer_pub_key = PublicKey::new(peer_pub_key_bytes);
            let ecdh_res = pr_key.ecdh(&peer_pub_key);
            assert!(matches!(ecdh_res, Err(())));
        }

        {
            let pr_key_hex = "77076D0A7318A57D3C16C17251B26645DF4C2F87EBC0992AB177FBA51DB92C2A";
            let peer_pub_key_hex = "5F9C95BCA3508C24B1D0B1559C83EF5B04445CC4581C8E86D8224EDDD09F1157";
            _test_x25519_failure_(pr_key_hex, peer_pub_key_hex);
        }

        {
            let pr_key_hex = "77076D0A7318A57D3C16C17251B26645DF4C2F87EBC0992AB177FBA51DB92C2A";
            let peer_pub_key_hex = "0100000000000000000000000000000000000000000000000000000000000000";
            _test_x25519_failure_(pr_key_hex, peer_pub_key_hex);
        }
        {
            let pr_key_hex = "77076D0A7318A57D3C16C17251B26645DF4C2F87EBC0992AB177FBA51DB92C2A";
            let peer_pub_key_hex = "ECFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF7F";
            _test_x25519_failure_(pr_key_hex, peer_pub_key_hex);
        }
        {
            let pr_key_hex = "77076D0A7318A57D3C16C17251B26645DF4C2F87EBC0992AB177FBA51DB92C2A";
            let peer_pub_key_hex = "5F9C95BCA3508C24B1D0B1559C83EF5B04445CC4581C8E86D8224EDDD09F1157";
            _test_x25519_failure_(pr_key_hex, peer_pub_key_hex);
        }
        {
            let pr_key_hex = "77076D0A7318A57D3C16C17251B26645DF4C2F87EBC0992AB177FBA51DB92C2A";
            let peer_pub_key_hex = "E0EB7A7C3B41B8AE1656E3FAF19FC46ADA098DEB9C32B1FD866205165F49B800";
            _test_x25519_failure_(pr_key_hex, peer_pub_key_hex);
        }
    }

    fn _test_x25519_success_(pr_key_hex: &str, peer_pub_key_hex: &str, ss_hex: &str) {
        let pr_key_bytes: [u8; 32] = hex_decode(pr_key_hex).try_into().unwrap();
        let pr_key = PrivateKey::new(pr_key_bytes);

        let expected_ss_bytes: [u8; 32] = hex_decode(ss_hex).try_into().unwrap();
        let peer_pub_key_bytes: [u8; 32] = hex_decode(peer_pub_key_hex).try_into().unwrap();
        let peer_pub_key = PublicKey::new(peer_pub_key_bytes);
        let ecdh_res = pr_key.ecdh(&peer_pub_key);
        assert!(matches!(ecdh_res, Ok(ss) if ss == expected_ss_bytes));
    }

    // X25519 test vector from RFC 7748, Section 6.1.
    #[test]
    fn test_x25519_rfc7748() {
        let pr_key_hex = "77076D0A7318A57D3C16C17251B26645DF4C2F87EBC0992AB177FBA51DB92C2A";
        let pub_key_hex = "8520F0098930A754748B7DDCB43EF75A0DBF3A0D26381AF4EBA4A98EAA9B4E6A";
        let peer_pub_key_hex = "DE9EDB7D7B7DC1B4D35B61C2ECE435373F8343C85B78674DADFC7E146F882B4F";
        let ss_hex = "4A5D9D5BA4CE2DE1728E3BF480350F25E07E21C947D19E3376F09B3C1E161742";

        _test_x25519_success_(pr_key_hex, peer_pub_key_hex, ss_hex);

        {
            let pr_key_bytes: [u8; 32] = hex_decode(pr_key_hex).try_into().unwrap();
            let pr_key = PrivateKey::new(pr_key_bytes);
            let expected_pub_key_bytes: [u8; 32] = hex_decode(pub_key_hex).try_into().unwrap();
            let pub_key = pr_key.public_key();
            assert_eq!(pub_key.public, expected_pub_key_bytes);
        }
    }

    // https://github.com/AdoptOpenJDK/openjdk-jdk/blob/master/test/jdk/sun/security/ec/xec/TestXDH.java
    #[test]
    fn test_x25519_01() {
        {
            let pr_key_hex = "A546E36BF0527C9D3B16154B82465EDD62144C0AC1FC5A18506A2244BA449AC4";
            let peer_pub_key_hex = "E6DB6867583030DB3594C1A424B15F7C726624EC26B3353B10A903A6D0AB1C4C";
            let ss_hex = "C3DA55379DE9C6908E94EA4DF28D084F32ECCF03491C71F754B4075577A28552";

            _test_x25519_success_(pr_key_hex, peer_pub_key_hex, ss_hex);
        }

        {
            let pr_key_hex = "4B66E9D4D1B4673C5AD22691957D6AF5C11B6421E0EA01D42CA4169E7918BA0D";
            let peer_pub_key_hex = "E5210F12786811D3F4B7959D0538AE2C31DBE7106FC03C3EFC4CD549C715A493";
            let ss_hex = "95CBDE9476E8907D7AADE45CB4B873F88B595A68799FA152E6F8F7647AAC7957";

            _test_x25519_success_(pr_key_hex, peer_pub_key_hex, ss_hex);
        }

        {
            let pr_key_hex = "77076D0A7318A57D3C16C17251B26645DF4C2F87EBC0992AB177FBA51DB92C2A";
            let peer_pub_key_hex = "FEFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF7F";
            let ss_hex = "81A02A45014594332261085128959869FC0540C6B12380F51DB4B41380DE2C2C";

            _test_x25519_success_(pr_key_hex, peer_pub_key_hex, ss_hex);
        }

        {
            let pr_key_hex = "77076D0A7318A57D3C16C17251B26645DF4C2F87EBC0992AB177FBA51DB92C2A";
            let peer_pub_key_hex = "DE9EDB7D7B7DC1B4D35B61C2ECE435373F8343C85B78674DADFC7E146F882B8F";
            let ss_hex = "954E472439316F118AE158B65619EECFF9E6BCF51AB29ADD66F3FD088681E233";

            _test_x25519_success_(pr_key_hex, peer_pub_key_hex, ss_hex);
        }

        {
            let pr_key_hex = "77076D0A7318A57D3C16C17251B26645DF4C2F87EBC0992AB177FBA51DB92C2A";
            let peer_pub_key_hex = "DE9EDB7D7B7DC1B4D35B61C2ECE435373F8343C85B78674DADFC7E146F882B4F";
            let ss_hex = "4a5d9d5ba4ce2de1728e3bf480350f25e07e21c947d19e3376f09b3c1e161742";

            _test_x25519_success_(pr_key_hex, peer_pub_key_hex, ss_hex);
        }

        {
            let pr_key_hex = "5DAB087E624A8A4B79E17F8B83800EE66F3BB1292618B6FD1C2F8B27FF88E0EB";
            let peer_pub_key_hex = "8520F0098930A754748B7DDCB43EF75A0DBF3A0D26381AF4EBA4A98EAA9B4E6A";
            let ss_hex = "4A5D9D5BA4CE2DE1728E3BF480350F25E07E21C947D19E3376F09B3C1E161742";

            _test_x25519_success_(pr_key_hex, peer_pub_key_hex, ss_hex);
        }
    }

    #[test]
    fn test_mul_001() {
        {
            let five: [u8; 32] = [
                5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ];
            let eight: [u8; 32] = [
                8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ];
            let fe5 = field::Element::from_le_bytes(five);
            let fe8 = field::Element::from_le_bytes(eight);
            let fe40 = field::Element::multiply(&fe5, &fe8);
            assert_eq!(fe40.0, 40);
            let mut inv8 = field::Element::invert(&fe8);
            inv8.reduce();
            let mut one = field::Element::multiply(&fe8, &inv8);
            one.reduce();
            assert_eq!(one, field::Element::ONE.clone());
        }

        {
            // 2^255 - 19
            let _bytes_25519_: [u8; 32] = [
                0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xed,
            ];

            {
                // 2^255 - 24
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

            {
                // 2^255 - 24
                let bytes_25524: [u8; 32] = [
                    0xe8, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f,
                ];
                let fe_25524 = field::Element::from_le_bytes(bytes_25524);

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
        }
    }
}
