# Importatnt References

### C language implementation
https://github.com/floodyberry/supercop/blob/master/crypto_sign/ed25519/ref10/sc_reduce.c


### Magic numbers that show up in the Ref10 implementation
https://github.com/str4d/ed25519-java/blob/e0ac35769db8553fb714b09f0d3f3d2b001fd033/src/net/i2p/crypto/eddsa/math/ed25519/Ed25519ScalarOps.java#L86
(Java language implementation by str4d.)


### The details
    1. q = 2^252 + q0 where q0 = 27742317777372353535851937790883648493.
    2. s11 is the coefficient of 2^(11 * 21), s23 is the coefficient of 2^(23 * 21) and _2^252 = 2^((23-11) * 21)__
    3. 2^252 congruent -q0 modulo q.
    4. -q0 = 666643 * 2^0 + 470296 * 2^21 + 654183 * 2^(2*21) - 997805 * 2^(3*21) + 136657 * 2^(4*21) - 683901 * 2^(5*21)
    5. s23 * 2^(23*21) = s23 * 2^(12*21) * 2^(11*21) = s3 * 2^252 * 2^(11*21) congruent
        
        s23 * (666643 * 2^0 + 470296 * 2^21 + 654183 * 2^(2*21) - 997805 * 2^(3*21) + 136657 * 2^(4*21) - 683901 * 2^(5*21)) * 2^(11*21) modulo q

        s23 * (666643 * 2^(11*21) + 470296 * 2^(12*21) + 654183 * 2^(13*21) - 997805 * 2^(14*21) + 136657 * 2^(15*21) - 683901 * 2^(16*21)).
            s11 += s23 * 666643;
            s12 += s23 * 470296;
            s13 += s23 * 654183;
            s14 -= s23 * 997805;
            s15 += s23 * 136657;
            s16 -= s23 * 683901;

The same procedure is then applied for s22,...,s18.


### Python snippet

```
limbs = [666643, 470296 * (2**21), 654183 * (2**42), -997805 * (2**63), 136657 * (2**84), -683901 * (2**105)]
assert(sum(limbs) == -27742317777372353535851937790883648493)
```

limbs = [666643, 470296 * (2**21), 654183 * (2**42), -997805 * (2**63), 136656 * (2**84), -364676 * (2**105)]
assert(sum(limbs) == -27742317777372353535851937790883648493)


limbs = [666643, 470296 * (2 ** 42), 654183 * (2 ** 84), -997805 * (2 ** 126), 136656 * (2 ** 168), 364675 * (2**210)]
assert(sum(limbs) == -27742317777372353535851937790883648493)