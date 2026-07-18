pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- Does literal/arithmetic quantization to the 16.16 grid truncate toward
-- zero or round to nearest? Use tostr(x,0x1) to see the raw fixed hex
-- bits directly, since default display only shows 4 decimal digits.
-- 0.1*65536=6553.6 -> truncate gives raw 6553, round gives 6554.
printh("RESULT lit_0_1 "..tostr(0.1,0x1))
-- 0.99999*65536=65535.34 -> truncate 65535, round 65535 (same either way)
-- so use a case just past the half-tick boundary instead:
-- 1.99999*65536=131071.34 -> truncate 131071, round 131071 (same)
-- 0.0000230*65536=1.507 -> truncate 1, round 2
printh("RESULT above_half_tick "..tostr(0.0000230,0x1))
-- 1/3*65536=21845.33 -> truncate 21845, round 21845 (same, need a case
-- with fraction >= .5): 7/3=2.3333, *65536=152917.33 -> same issue.
-- 5/6*65536=54613.33 -> same. Use 1/6: 0.16666667*65536=10922.667
-- truncate 10922, round 10923.
printh("RESULT div_1_6 "..tostr(1/6,0x1))
printh("RESULT neg_div_1_6 "..tostr(-1/6,0x1))
printh("DONE")
