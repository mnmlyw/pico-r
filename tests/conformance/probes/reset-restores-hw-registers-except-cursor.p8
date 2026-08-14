pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- reset() restores the hardware-register defaults across 0x5F00..0x5F7F,
-- with one measured exception: the print cursor's three bytes -- home-x
-- (0x5F24) and x/y (0x5F26/0x5F27) -- keep whatever the cart left there.
--
-- 0x5F44..0x5F4B is DELIBERATELY not covered: that is the console's
-- memory-mapped RNG state, it changes run to run, and goldening it would
-- capture a coin flip. pico-r keeps its PRNG outside RAM so it reads zero
-- there -- a separate gap, noted rather than papered over.
--
-- Defaults worth calling out because they are not zero and not derivable:
-- 0x5F25 colour is 6, clip right/bottom are 128, 0x5F55/0x5F56/0x5F57 are
-- 0x60/0x20/128, 0x5F5E (bitplane mask) is 0xFF, and 0x5F60..0x5F6F is a
-- fixed non-identity secondary palette that is factory data.
for a=0x5f00,0x5f7f do poke(a,0xa5) end
reset()
for b=0,7 do
	s=""
	for i=0,15 do
		local a=0x5f00+b*16+i
		if a>=0x5f44 and a<=0x5f4b then
			s=s.."-- "
		else
			s=s..peek(a).." "
		end
	end
	printh("RESULT row"..b.." "..s)
end
-- the cursor bytes specifically: dirtied to 0xa5, still 0xa5 after reset
printh("RESULT cursor "..peek(0x5f24).." "..peek(0x5f26).." "..peek(0x5f27))
-- reset() leaves GPIO and the screen alone
poke(0x5f80,0x5a) poke(0x6000,0x5a) reset()
printh("RESULT untouched "..peek(0x5f80).." "..peek(0x6000))
printh("DONE")
