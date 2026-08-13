pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- tonum's flag field is NOT the same rule as tostr's, despite the identical
-- spelling. Three separate divergences, all still open:
--   * a "0x...." string parses as a hex literal with NO flag (pico-r: nil)
--   * flag 1 means "read the digits as hexadecimal", so "12" -> 0x12 -> 18,
--     and "0x0001.0000" is then not valid hex digits and yields 0
--     (pico-r returns 1, as if it had parsed the literal)
--   * a BOOLEAN flag is IGNORED here -- tonum(s,true) == tonum(s) -- which is
--     the opposite of tostr, where true means flag 1 (pico-r returns nil)
function q(l,v) printh("RESULT "..l.." "..tostr(v)) end
q("hexstr_none", tonum("0x0001.0000"))
q("hexstr_f1",   tonum("0x0001.0000",1))
q("hexstr_true", tonum("0x0001.0000",true))
q("dec_none",    tonum("12"))
q("dec_f1",      tonum("12",1))
q("dec_true",    tonum("12",true))
q("bad_none",    tonum("zz"))
q("bad_f4",      tonum("zz",4))
printh("DONE")
