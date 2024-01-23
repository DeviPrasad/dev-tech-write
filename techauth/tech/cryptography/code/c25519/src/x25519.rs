use crate::field;
use crate::subtle;

// scalar: little endian sequence of bytes
// point: little endian sequence of bytes
pub fn scalar_mul(scalar: [u8; 32], point: [u8; 32], dst: &mut [u8; 32]) {
    let mut e: [u8; 32] = [0; 32];

    e.copy_from_slice(&scalar);
    e[0] &= 248;
    e[31] &= 127;
    e[31] |= 64;

    let x1 = field::Element::from_le_bytes(point);
    let mut x2 = field::Element::ONE.clone();
    let mut x3 = x1.clone();
    let mut z2 = field::Element::ZERO.clone();
    let mut z3 = field::Element::ONE.clone();
    let mut swap: u32 = 0;
    for pos in (0..=254).rev() {
        let b: u32 = (e[pos/8] >> (pos & 7)) as u32;
        let b = b & 1;
        swap ^= b;
        field::Element::swap(&mut x2, &mut x3, swap);
        field::Element::swap(&mut z2, &mut z3, swap);
        swap = b;

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
        let x25519_base_point: [u8; 32]= [
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
    use crate::x25519::{PrivateKey, PublicKey};

    fn hex_decode(s: &str) -> Vec<u8> {
        let r = hex::decode(s);
        assert!(r.is_ok());
        r.unwrap()
    }

    fn test_x25519_failure(private: [u8; 32], public: [u8; 32]) {
        let pr_key = PrivateKey::new(private);
        let pub_key = PublicKey::new(public);
        let res = pr_key.ecdh(&pub_key);
        assert!(matches!(res, Err(())));
    }

    #[test]
    fn test_failure() {
        let identity: &[u8] = &hex_decode("0000000000000000000000000000000000000000000000000000000000000000");
        let low_order_point: &[u8] = &hex_decode("e0eb7a7c3b41b8ae1656e3faf19fc46ada098deb9c32b1fd866205165f49b800");
        let mut random_scalar: [u8; 32] = [0; 32];

        rand::thread_rng().fill(&mut random_scalar);
        test_x25519_failure(random_scalar, identity.try_into().unwrap());
        test_x25519_failure(random_scalar, low_order_point.try_into().unwrap());
    }

    // X25519 test vector from RFC 7748, Section 6.1.
    #[test]
    fn test_x25519_rfc7748() {
        let pr_key_hex = "77076d0a7318a57d3c16c17251b26645df4c2f87ebc0992ab177fba51db92c2a";
        let pub_key_hex = "8520f0098930a754748b7ddcb43ef75a0dbf3a0d26381af4eba4a98eaa9b4e6a";
        let peer_pub_key_hex = "de9edb7d7b7dc1b4d35b61c2ece435373f8343c85b78674dadfc7e146f882b4f";
        let ss_hex = "4a5d9d5ba4ce2de1728e3bf480350f25e07e21c947d19e3376f09b3c1e161742";

        let pr_key_bytes: [u8; 32] = hex_decode(pr_key_hex).try_into().unwrap();
        let pr_key = PrivateKey::new(pr_key_bytes);

        let expected_pub_key_bytes: [u8; 32] = hex_decode(pub_key_hex).try_into().unwrap();
        let pub_key = pr_key.public_key();
        assert_eq!(pub_key.public, expected_pub_key_bytes);

        let expected_ss_bytes: [u8; 32] = hex_decode(ss_hex).try_into().unwrap();
        let peer_pub_key_bytes: [u8; 32] = hex_decode(peer_pub_key_hex).try_into().unwrap();
        let peer_pub_key = PublicKey::new(peer_pub_key_bytes);
        let ecdh_res = pr_key.ecdh(&peer_pub_key);
        assert!(matches!(ecdh_res, Ok(ss) if ss == expected_ss_bytes));
    }
}
