# Is X25519 Associative? Sometimes!
https://words.filippo.io/dispatches/x25519-associative/
27 May 2020

Curve25519 has cofactor 8, meaning that you can think of points as having two independent values, the first of which has order q, the second of order 8. Multiplying a point by a multiple of 8 will ensure the result has a value of zero in the cofactor component (8k * n = 0 mod 8), avoiding leaking the value of the scalar modulo 8. It's enough to make a curve with a cofactor tenable for authenticated Diffie-Hellman, but it's not a safe design and it has caused a bunch of real-world bugs. New cryptosystems should just use a group of pure prime order, like ristretto255.

Monero Documentation
Edwards25519 Elliptic Curve
https://monerodocs.org/cryptography/asymmetric/edwards25519/


https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-186.pdf
Recommendations for Discrete Logarithm-based Cryptography:
Elliptic Curve Domain Parameters
February 2023





CFRG Elliptic Curve Diffie-Hellman (ECDH) and Signatures in JSON Object Signing and Encryption (JOSE)
https://www.rfc-editor.org/rfc/rfc8037.html#page-4


https://www.iana.org/assignments/jose/jose.xhtml
SON Web Key Elliptic Curve

Curve Name 	Curve Description 	JOSE Implementation Requirements 	Change Controller 	Reference 
P-256 	    P-256 Curve 	    Recommended+ 	[IESG] 	[RFC7518, Section 6.2.1.1]
P-384 	    P-384 Curve 	    Optional 	[IESG] 	[RFC7518, Section 6.2.1.1]
P-521 	    P-521 Curve 	    Optional 	[IESG] 	[RFC7518, Section 6.2.1.1]
Ed25519 	Ed25519 signature algorithm key pairs 	Optional 	[IESG] 	[RFC8037, Section 3.1]
Ed448 	    Ed448 signature algorithm key pairs 	Optional 	[IESG] 	[RFC8037, Section 3.1]
X25519 	    X25519 function key pairs 	Optional 	[IESG] 	[RFC8037, Section 3.2]
X448 	    X448 function key pairs 	Optional 	[IESG] 	[RFC8037, Section 3.2]


