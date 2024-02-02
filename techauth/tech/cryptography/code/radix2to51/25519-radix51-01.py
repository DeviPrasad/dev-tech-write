
# 5 limbs
# limb r51_a4 does not look good; it is > (pow(2, 51) - 1)
# therefore, the values r51_a1 t0 r51_a4 need to be reduced
# so 
def arith_radix51_25519():
    r51_a0 = 0x7FFFFFFFFFFDA
    r51_a1 = 0x7FFFFFFFFFFFF
    r51_a2 = 0x7FFFFFFFFFFFF
    r51_a3 = 0x7FFFFFFFFFFFF
    r51_a4 = 0xFFFFFFFFFFFFF
    #
    assert r51_a0 < pow(2, 51) - 1
    assert r51_a1 <= pow(2, 51) - 1
    assert r51_a2 <= pow(2, 51) - 1
    assert r51_a3 <= pow(2, 51) - 1
    assert r51_a4 <= pow(2, 52) - 1 # note the different invariant
    #
    norm_a0 = r51_a0
    norm_a1 = pow(2, 51) * r51_a1
    norm_a2 = pow(2, 102) * r51_a2
    norm_a3 = pow(2, 153) * r51_a3
    norm_a4 = pow(2, 204) * r51_a4
    #
    assert norm_a4 | norm_a3 | norm_a2 | norm_a1 | norm_a0 == pow(2,256) - 38
    assert norm_a0 | norm_a1 | norm_a2 | norm_a3 | norm_a4 == pow(2,256) - 38
    #    
    print(hex(norm_a4 | norm_a3 | norm_a2 | norm_a1 | norm_a0))
    print(hex(norm_a0 | norm_a1 | norm_a2 | norm_a3 | norm_a4))
    #
    print("curve25519 unverified version 01")
    print("a0 =", hex(norm_a0))
    print("a1 =", hex(norm_a1))
    print("a2 =", hex(norm_a2))
    print("a3 =", hex(norm_a3))
    print("a4 =", hex(norm_a4))
    print("a2|3|a4 =", hex(norm_a2 |norm_a3 | norm_a4))

arith_radix51_25519()



# 6 limbs
# correct answer
# although the assertions hold, this is not 2^255-19
def arith_radix51_25519():
    r51_a0 = 0x7FFFFFFFFFFDA
    r51_a1 = 0x7FFFFFFFFFFFF
    r51_a2 = 0x7FFFFFFFFFFFF
    r51_a3 = 0x7FFFFFFFFFFFF
    r51_a4 = 0x7FFFFFFFFFFFF
    r51_a5 = 0x1
    #
    assert r51_a0 < pow(2, 51) - 1
    assert r51_a1 <= pow(2, 51) - 1
    assert r51_a2 <= pow(2, 51) - 1
    assert r51_a3 <= pow(2, 51) - 1
    assert r51_a4 <= pow(2, 51) - 1
    assert r51_a5 <= pow(2, 51) - 1
    #
    norm_a0 = r51_a0
    norm_a1 = pow(2, 51) * r51_a1
    norm_a2 = pow(2, 102) * r51_a2
    norm_a3 = pow(2, 153) * r51_a3
    norm_a4 = pow(2, 204) * r51_a4
    norm_a5 = pow(2, 255) * r51_a5
    #
    assert norm_a5 | norm_a4 | norm_a3 | norm_a2 | norm_a1 | norm_a0 == pow(2,256) - 38
    assert norm_a0 | norm_a1 | norm_a2 | norm_a3 | norm_a4 | norm_a5 == pow(2,256) - 38
    #    
    print(norm_a5 | norm_a4 | norm_a3 | norm_a2 | norm_a1 | norm_a0)
    print(norm_a0 | norm_a1 | norm_a2 | norm_a3 | norm_a4 | norm_a5)
    #
    print("a0 =", hex(norm_a0))
    print("a5 =", hex(norm_a5))
    print("a4|a5 =", hex(norm_a4 | norm_a5))

arith_radix51_25519()




import collections

def radix51(n):
    num = n
    i = 51
    rems = collections.deque()
    divs = collections.deque()
    while num >= 1:
        _num_ = num
        rem = num % pow(2, i)
        dv = num // pow(2, i)
        #num = (num // pow(2, i))
        #num = (num >> 51) << 51
        num = (num >> 51)
        #assert _num_ == num * pow(2, i) + rem
        rems.appendleft(hex(rem))
        divs.appendleft(hex(dv))
        #print(hex(num), hex(rem))
        #print(rems)
        #i = i + 51
    #
    print()
    print("rems =", rems)
    print("divs =", divs)
    print()
    return rems


radix51(pow(2,256)-38)


output:
['0x1', '0x7ffffffffffff', '0x7ffffffffffff', '0x7ffffffffffff', '0x7ffffffffffff', '0x7ffffffffffda']


0x8000000000000
0x8000000000000
0x7ffffffffffda
0x7ffffffffffda
0xFFFFFFFFFFFDA

hex((pow(2,256)-38) % pow(2,51))

0x7ffffffffffff
0xfffffffffffff
0b1000000000000000000000000000000000000000000000000000
 0b111111111111111111111111111111111111111111111111111

0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffda
0x8000000000000000000000000000000000000000000000000000000000000000
