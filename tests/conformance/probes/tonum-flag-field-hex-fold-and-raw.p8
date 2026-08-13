pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- tonum's second argument is a 3-bit field (floor'd, two's-complement -- a
-- flag of -1 reads as 7; bits above 0x4 do nothing):
--   0x1  parse as hex: a FOLD over EVERY character, acc = acc*16 + digit,
--        wrapping mod 2^32, where any non-hex character counts as digit
--        ZERO (space, '.', 'x', '-' included: "  7 " reads as 0x0070 = 112,
--        "1.5" as 0x105, "0b101" as 0xb101). Case-insensitive. Never fails.
--   0x2  result is RAW 16.16 bits instead of an integer value. Without 0x1
--        it is a decimal-integer scan (spaces, one sign, digits to the
--        first non-digit) -- so "1e3" under it is 1, not 1000. Never fails.
--   0x4  failed parse gives 0 instead of nil -- STRING failures only.
-- Results printed as raw hex so this tests tonum, not the decimal renderer
-- (which has its own divergence on tiny values -- see the LEDGER).
function q(l,v) printh("RESULT "..l.." "..(v==nil and "nil" or tostr(v,3))) end
for _,s in pairs({"12","-12","1.5","0x0001.0000","0x12","ff","-ff","","  7 ",
                  "zz","1e3","0b101","32768",".5","+3","FF","0Xff","--3","- 3",
                  "2e38","0B101","1e-2"}) do
	for f=0,7 do q("s_"..s.."_f"..f, tonum(s,f)) end
end
-- non-strings: numbers pass through unchanged for EVERY flag; booleans are
-- 1/0 with 0x2 making that the raw value; nil/table are nil even under 0x4
q("num_f2", tonum(5,2))
q("negnum_f3", tonum(-1.5,3))
q("true_f0", tonum(true,0))
q("true_f2", tonum(true,2))
q("false_f2", tonum(false,2))
q("nil_f4", tonum(nil,4))
q("tbl_f4", tonum({},4))
-- flag coercion: floor, two's-complement &7, booleans/garbage read as 0
q("flag_neg1", tonum("12",-1))
q("flag_1p9", tonum("12",1.9))
q("flag_true", tonum("12",true))
q("flag_zz", tonum("12","zz"))
printh("DONE")
