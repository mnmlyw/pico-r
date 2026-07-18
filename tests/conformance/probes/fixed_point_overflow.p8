pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- 16.16 fixed-point overflow/wraparound in plain arithmetic. Real PICO-8
-- numbers are a 32-bit signed fixed value (16 int bits, 16 frac bits);
-- results that exceed the representable range wrap modulo 2^32 rather
-- than promoting to a wider type.
printh("RESULT add_over_top "..(32767.5+1))
printh("RESULT add_over_top2 "..(32767+1))
printh("RESULT mul_wrap "..(20000*2))
printh("RESULT mul_wrap_neg "..(-20000*2))
printh("RESULT neg_min "..(-(-32768)))
printh("RESULT sub_under_bottom "..(-32768-1))
printh("RESULT big_mul "..(1000*1000))
printh("RESULT max_lit_plus_eps "..(32767.99998+0.00002))
printh("DONE")
