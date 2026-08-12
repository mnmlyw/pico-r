pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
x = 1 >>> 16 - 8
printh("RESULT shr_arith_space "..x)
y = 12 & 15 - 1
printh("RESULT band_arith_space "..y)
z = 1 << 4 - 1
printh("RESULT shl_arith_space "..z)
w = 1 >>> 16-8
printh("RESULT shr_arith_nospace "..w)
printh("DONE")
