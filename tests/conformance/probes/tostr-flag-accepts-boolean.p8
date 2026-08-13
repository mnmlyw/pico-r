pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- tostr's second argument is a 2-bit FORMAT FIELD, not a boolean:
--   0 decimal value | 1 hex value "0xHHHH.HHHH"
--   2 decimal raw   | 3 hex raw "0xHHHHHHHH"
-- Bits above those two are ignored.
--
-- But `tostr(x,true)` is the documented spelling of the hex flag and the one
-- real carts actually use, and a boolean is not a number -- so the ordinary
-- numeric-argument path drops it and formats as plain decimal. Confirmed:
-- true means 1 and false means 0.
for _,n in pairs({0,1,-1,0.5,-0.5,32767,-32768,100.25}) do
	printh("RESULT n"..n.." "..tostr(n).." "..tostr(n,1).." "..tostr(n,2).." "..tostr(n,3))
	printh("RESULT b"..n.." "..tostr(n,true).." "..tostr(n,false))
end
-- higher bits are ignored, not additive
printh("RESULT high "..tostr(-1.5,4).." "..tostr(-1.5,5).." "..tostr(-1.5,7))
-- the flag has no effect on non-numbers
printh("RESULT str "..tostr("ab",true).." bool "..tostr(true,true).." nil "..tostr(nil,true))
printh("DONE")
