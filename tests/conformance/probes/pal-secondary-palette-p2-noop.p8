pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
srand(1)

local function hash()
	local h=0
	for i=0,8191 do h=(h+peek(0x6000+i)*(i+1))%99999999 end
	return h
end

-- A: secondary palette set, then a two-color fillp pattern used to draw.
pal()
cls()
pal(12,0x87,2)
fillp(0b0011001111001100.1)
circfill(20,20,10,12)
printh("RESULT screen_hash_with_p2 "..hash())

-- B: control -- same pattern/draw call, no secondary palette set.
pal()
cls()
fillp(0b0011001111001100.1)
circfill(20,20,10,12)
printh("RESULT screen_hash_without_p2 "..hash())

-- C: secondary palette set, but no fill pattern active (solid fill).
pal()
cls()
pal(12,0x87,2)
fillp(0)
circfill(20,20,10,12)
printh("RESULT screen_hash_p2_no_pattern "..hash())

-- D: table form of pal(...,2).
pal()
cls()
pal({[3]=9},2)
fillp(0b0011001111001100.1)
circfill(40,40,10,3)
printh("RESULT screen_hash_p2_table "..hash())

-- E: does pal() (full reset) clear whatever p2 set, so D-style draw after
-- a reset behaves like the no-p2 control?
pal()
cls()
pal({[3]=9},2)
pal()
fillp(0b0011001111001100.1)
circfill(40,40,10,3)
printh("RESULT screen_hash_after_reset "..hash())

-- F: direct memory probe of the documented secondary-palette storage area.
pal()
pal(12,0x87,2)
printh("RESULT peek_0x5f60_plus_12 "..peek(0x5f60+12))
printh("RESULT peek_0x5f70_plus_12 "..peek(0x5f70+12))

printh("DONE")
