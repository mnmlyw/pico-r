pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- tostr() format-flag second argument: 0x1 hex "0xHHHH.LLLL", 0x2 raw
-- 32-bit fixed value as plain decimal, 0x3 hex digits with no "." --
-- plus the no-args-vs-explicit-nil distinction.
printh("RESULT hex_pos "..tostr(1.5,0x1))
printh("RESULT hex_neg "..tostr(-1.5,0x1))
printh("RESULT hex_zero "..tostr(0,0x1))
printh("RESULT raw_pos "..tostr(1,0x2))
printh("RESULT raw_neg "..tostr(-1,0x2))
printh("RESULT raw_frac "..tostr(1.5,0x2))
printh("RESULT combined "..tostr(1.5,0x3))
printh("RESULT no_args_len "..#tostr())
printh("RESULT nil_val "..tostr(nil))
printh("RESULT bool_true "..tostr(true))
printh("RESULT bool_false "..tostr(false))
printh("DONE")
