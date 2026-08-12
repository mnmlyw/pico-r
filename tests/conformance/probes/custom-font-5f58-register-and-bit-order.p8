pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- Two things about the custom font at 0x5600.
--
-- 1. GLYPH BIT ORDER is LSB-first: bit 0 is the LEFTMOST column.
--    An earlier round recorded this as MSB-first and cited an "asymmetric
--    per-row bit pattern (0x81)" as the evidence -- but 0x81 is 10000001,
--    symmetric under bit reversal, so that test could not distinguish the
--    two orders and locked in the wrong one. The rows below are genuinely
--    asymmetric: a row of 0x01 must light ONLY the leftmost pixel and a row
--    of 0x80 ONLY the rightmost.
--
-- 2. The font is selectable WITHOUT any control code, via the 0x5F58
--    register. Swept all 256 values: the custom font is used exactly when
--    bit 7 is set, bit 5 is CLEAR and bit 0 is set -- (reg & 0xA1) == 0x81.
--    Bit 5 vetoing an otherwise-enabled register is the non-obvious part.
--    ascent-0 drives this with poke(0x5f58,0x81).
poke(0x5600,8) poke(0x5602,8)
poke(0x5600+65*8+0,0x01)
poke(0x5600+65*8+1,0x80)
poke(0x5600+65*8+2,0x0f)

function row(y)
	local s=""
	for x=0,7 do s=s..(pget(x,y)!=0 and "#" or ".") end
	return s
end

-- via the 0x5F58 register
cls(0) poke(0x5f58,0x81) print("A",0,0,7) poke(0x5f58,0)
printh("RESULT reg_row0 "..row(0))
printh("RESULT reg_row1 "..row(1))
printh("RESULT reg_row2 "..row(2))

-- via the \014 control code: must agree with the register path
cls(0) print("\014A",0,0,7)
printh("RESULT ctl_row0 "..row(0))
printh("RESULT ctl_row1 "..row(1))
printh("RESULT ctl_row2 "..row(2))

-- the enable predicate, including the bit-5 veto and the bit-6 don't-care.
-- A blank glyph is a transform-proof probe: it stays blank under any width
-- or scaling the other bits might apply, so "drew nothing" means the custom
-- font really is live rather than merely shifted.
for i=0,7 do poke(0x5600+66*8+i,0) end
for _,v in pairs({0x00,0x01,0x80,0x81,0x83,0xa1,0xc1,0xe1,0x9f,0xdf,0xff}) do
	cls(0)
	poke(0x5f58,v)
	print("B",0,0,7)
	poke(0x5f58,0)
	local blank=true
	for y=0,15 do
		for x=0,23 do
			if pget(x,y) != 0 then blank=false end
		end
	end
	printh("RESULT enable_"..v.." custom="..tostr(blank))
end

printh("DONE")
