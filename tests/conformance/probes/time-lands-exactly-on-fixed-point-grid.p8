pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- t()/time() must be an EXACT rational number of frames over the frame rate,
-- quantized to the 16.16 grid by truncation -- not a running floating-point
-- sum of 1/30.
--
-- 1/30 is not representable in binary floating point, so accumulating it
-- leaves the total a hair BELOW the true value. That is invisible almost
-- everywhere, because truncating to 16.16 throws the error away anyway. It
-- is NOT invisible at the frames where the true value sits exactly ON the
-- grid: there the sum is one ULP short and truncation drops a whole step.
-- At 30fps that is every 15th frame -- t()==0.5 reads back 0x0000.7fff
-- instead of 0x0000.8000, t()==1.0 reads back 0x0000.ffff instead of
-- 0x0001.0000, and so on forever.
--
-- Printed as hex so the comparison is on the exact 16.16 bits rather than a
-- decimal rendering that would hide a one-ULP difference. A top-level flip()
-- loop is used because the conformance harness runs each probe for a single
-- _update frame, which would never reach the interesting frames.
f=0
while true do
	printh("RESULT t_flip"..f.." "..tostr(t(),1))
	if f >= 60 then
		break
	end
	f += 1
	flip()
end
printh("DONE")
