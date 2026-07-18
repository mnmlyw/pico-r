pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- poke4() stores a number as raw 16.16 fixed bits. Does an out-of-range
-- value wrap (mod 2^32, matching to_fixed's bitwise ops) or clamp to
-- i32::MIN/MAX when re-read via peek4?
poke4(0,100000)
printh("RESULT poke4_over_pos "..peek4(0))
poke4(0,-100000)
printh("RESULT poke4_over_neg "..peek4(0))
poke4(0,32767.99998)
printh("RESULT poke4_max_ok "..peek4(0))
poke4(0,2^40)
printh("RESULT poke4_huge "..peek4(0))

cartdata("fixedpointprobe")
dset(0,100000)
printh("RESULT dget_over_pos "..dget(0))
dset(0,-100000)
printh("RESULT dget_over_neg "..dget(0))
printh("DONE")
