pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- line() is not Bresenham. Endpoints swap so Y ascends -- which makes the
-- rendering symmetric under endpoint reversal, where Bresenham's two
-- directions disagree -- then a 16.16 DDA walks the major axis, stepping
-- the minor accumulator from a +0.5 bias by the C-truncated 16.16 slope
-- and plotting its floor.
--
-- Discriminating cases pinned here:
--  * (0,0)->(12,6): exact math puts x=6 at 3.0; the console plots 2,
--    because the accumulated truncated slope reaches only 2.99994.
--  * (33,42)->(22,56): steep, descending x; y=49 plots x=28, which only
--    the toward-zero quotient gives (a floored step gives 27).
--  * each reversal must plot the identical set.
function dump(i,...)
	cls(0)
	line(...)
	local s=""
	for y=0,127 do for x=0,127 do
		if (pget(x,y)!=0) s=s..x..","..y..";"
	end end
	printh("RESULT c"..i.." "..s)
end
dump(1, 0,0,12,5,7)
dump(2, 12,5,0,0,7)
dump(3, 0,0,5,12,7)
dump(4, 5,12,0,0,7)
dump(5, 0,0,12,6,7)
dump(6, 12,6,0,0,7)
dump(7, 0,5,12,0,7)
dump(8, 33,42,22,56,7)
dump(9, 22,56,33,42,7)
dump(10, 3,3,3,3,7)
dump(11, 2,7,9,7,7)
dump(12, 7,2,7,9,7)
dump(13, 0.9,0.9,11.9,4.9,7)
-- long lines, where the truncated slope's error visibly accumulates
dump(14, 0,0,127,43,7)
dump(15, 127,43,0,0,7)
dump(16, 5,127,119,0,7)
printh("DONE")
