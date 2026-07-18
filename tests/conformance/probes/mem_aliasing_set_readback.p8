pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- Does sset() even affect the shared region at all (sget readback), or
-- does it get silently clipped/blocked for sprite rows >= 64?
sset(0,64,5)
printh("RESULT sget_after_sset "..sget(0,64))
printh("RESULT peek_after_sset "..peek(0x1000))
printh("RESULT mget_after_sset "..mget(0,32))

-- and the reverse: does mset() affect the shared region / sget readback?
mset(1,32,42)
printh("RESULT peek_after_mset "..peek(0x1001))
printh("RESULT sget_after_mset_x2 "..sget(2,64))
printh("RESULT sget_after_mset_x3 "..sget(3,64))
printh("RESULT mget_after_mset "..mget(1,32))
printh("DONE")
