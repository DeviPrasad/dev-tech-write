
Guide to Elliptic Curve Cryptography

Binary fields (Page 26)
Example 2.2 (binary field F24 ) The elements of F24 are the 16 binary polynomials of degree at most 3:


k.<z> = GF(2^4)

k.characteristic()
2

k.order()
16


k.modulus()
x^4 + x + 1


k.from_integer(13)
z^3 + z^2 + 1


k.from_integer(7)
z^2 + z + 1


k.from_integer(13) + k.from_integer(7)
z^3 + z


k.from_integer(13) * k.from_integer(7)
z^2 + 1


1/k.from_integer(13)
z^2



