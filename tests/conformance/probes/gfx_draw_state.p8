pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- palt() single-bitmask arg: bit (15-k) controls color k (bit 15 = color
-- 0, bit 0 = color 15), not bit k directly.
palt(0b1000000000000000)
printh("RESULT palt_c0 "..peek(0x5f00+0))
printh("RESULT palt_c15 "..peek(0x5f00+15))
palt()

-- line() endpoint is memory-mapped at 0x5f3c/0x5f3e (16-bit each).
line(10,10,50,60)
printh("RESULT line_x "..peek2(0x5f3c))
printh("RESULT line_y "..peek2(0x5f3e))

-- pal(c0,c1,2) (secondary/pen palette) must not touch the draw palette.
pal()
pal(5,8,2)
printh("RESULT pal_p2_draw_unchanged "..peek(0x5f00+5))

-- pal({[k]=v}) wraps out-of-range keys via % 16.
pal()
pal({[20]=5})
printh("RESULT pal_table_oob "..peek(0x5f00+4))
pal()

printh("DONE")
