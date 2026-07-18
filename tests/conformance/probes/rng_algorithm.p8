pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- Pins down the exact rnd()/srand() algorithm: srand(0), a few explicit
-- seeds, and negative/zero n. (Deliberately excludes the never-seeded
-- default state and rnd(table) selection order -- both confirmed
-- oracle-side to not have a fixed target: the default state is seeded from
-- wall-clock time on real hardware (non-reproducible run-to-run, verified
-- directly), and rnd(table)'s exact index-selection formula wasn't pinned
-- down despite extensive attempts; see LEDGER.md.)
srand(0)
printh("RESULT srand0_rnd1 "..tostr(rnd(1),0x1))
printh("RESULT srand0_rnd2 "..tostr(rnd(1),0x1))

srand(12345)
printh("RESULT s12345_rnd1 "..tostr(rnd(1),0x1))
printh("RESULT s12345_rnd2 "..tostr(rnd(1),0x1))
printh("RESULT s12345_rnd1000 "..tostr(rnd(1000),0x1))

srand(1)
printh("RESULT s1_rnd1 "..tostr(rnd(1),0x1))

srand(-500)
printh("RESULT sneg500_rnd1 "..tostr(rnd(1),0x1))

srand(99)
printh("RESULT noarg_rnd "..tostr(rnd(),0x1))
printh("RESULT zero_rnd "..tostr(rnd(0),0x1))
printh("RESULT neg_rnd "..tostr(rnd(-10),0x1))
printh("DONE")
