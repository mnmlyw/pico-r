pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- tonum() format-flag second argument: 0x1 hex parse (inverse of tostr's
-- hex format), 0x2 raw fixed-bits reinterpretation, 0x4 return 0 instead
-- of nil on parse failure.
printh("RESULT hex_flag "..tostr(tonum("008e",0x1)))
printh("RESULT hex_flag_prefixed "..tostr(tonum("0x8e",0x1)))
printh("RESULT plain "..tostr(tonum("1.5")))
local a=tonum("notanum")
printh("RESULT type_fail "..type(a))
local b=tonum("notanum",0x4)
printh("RESULT type_fail_flag4 "..type(b).." val="..tostr(b))
local c=tonum("65536",0x2)
printh("RESULT raw_flag_str "..tostr(c))
local d=tonum(65536,0x2)
printh("RESULT raw_flag_num "..tostr(d))
printh("DONE")
