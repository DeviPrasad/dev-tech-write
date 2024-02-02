
Guide to Elliptic Curve Cryptography

Binary fields (Page 26)
Example 2.2 (binary field F24 ) The elements of F24 are the 16 binary polynomials of degree at most 3:


Extension Fields

Example 2.4 (an extension field) Let p = 251 and m = 5.

f_p251_m5.<z> = GF(251^5)


k2.<u>=f_p251_m5.extension(x^5+x^4+12*x^3+9*x^2+7)
k2
Univariate Quotient Polynomial Ring in u over Finite Field in z of size 251^5 with modulus u^5 + u^4 + 12*u^3 + 9*u^2 + 7


k2.modulus()
u^5 + u^4 + 12*u^3 + 9*u^2 + 7

u.minpoly()
u^5 + u^4 + 12*u^3 + 9*u^2 + 7

k2.characteristic()
251


k2.order()
981393183197870664268660488318325947986612503106090643756251

