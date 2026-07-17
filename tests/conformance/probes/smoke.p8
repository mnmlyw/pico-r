pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- harness smoke test: numeric + gfx readback via printh
printh("RESULT flr_neg "..flr(-3.5))
printh("RESULT shl_ovf "..(0x4000<<2))
cls(0)
circfill(64,64,10,8)
local h=0
for a=0x6000,0x7fff do h=(h*31+@a)&0x7fff end
printh("RESULT screen_hash "..h)
printh("DONE")
