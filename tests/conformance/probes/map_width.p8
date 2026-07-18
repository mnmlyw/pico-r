pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- The big-map width register (0x5f57): a raw 0 means width 256, not 128
-- -- confirmed against official PICO-8. A fresh cart must still default
-- to the normal 128-wide map (the register itself defaults to the byte
-- 128, not 0).
mset(50,10,3)
printh("RESULT default_write "..mget(50,10))
mset(150,10,4)
printh("RESULT beyond_default_width "..mget(150,10))

poke(0x5f57,0)
mset(200,10,7)
printh("RESULT width0_wide_write "..mget(200,10))

printh("DONE")
