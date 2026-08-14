pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- cartdata(id) returns whether a save for that id ALREADY EXISTED, and
-- (re)loads the 256-byte window at 0x5E00 from it. A fresh save therefore
-- reports FALSE and the window reads as zeros.
--
-- Within one run repeated calls all report false -- the file is only written
-- back at exit -- and each call re-zeroes the region, discarding any dset()
-- made before it. That last part is the surprising one and is pinned below.
--
-- This probe runs in a fresh -home every time, so "false" is the correct and
-- stable expectation. The console returns TRUE on a second run against the
-- same home; reproducing that needs a filesystem-backed store, which pico-r
-- does not have (see LEDGER).
printh("RESULT first "..tostr(cartdata("picor_probe_cd")))
printh("RESULT fresh_dget0 "..tostr(dget(0)))
dset(0,42) dset(63,7)
printh("RESULT after_dset "..dget(0).." "..dget(63))
-- dset writes the 16.16 raw little-endian into 0x5E00 + i*4
printh("RESULT ram "..peek(0x5e00).." "..peek(0x5e01).." "..peek(0x5e02).." "..peek(0x5e03))
-- a second call re-reads the (still absent) save, wiping what was just set
printh("RESULT recall "..tostr(cartdata("picor_probe_cd")).." dget0="..dget(0))
-- out-of-range indices read 0 and do not error
printh("RESULT oob "..tostr(dget(64)).." "..tostr(dget(-1)))
printh("DONE")
