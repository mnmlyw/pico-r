pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- probe: does fillp checkerboard/secondary-color dithering apply to
-- spr/sspr/map/tline the same way it does for shape draws (circ/rect/etc)?
-- manual says fillp only affects circ/rect/oval/pset/line by default, and
-- affects spr-family draws only when bit 0b0.010 (0x1000 in the 16-bit
-- pattern word) is set, in which case it uses the *secondary palette* --
-- not the color's high nibble like shape fills do.
for y=0,7 do
 for x=0,7 do
  sset(x,y,8)
 end
end
-- also fill a second sprite region solid, for map()/tline() coverage
for y=0,7 do
 for x=8,15 do
  sset(x,y,8)
 end
end
mset(0,0,0)
mset(1,0,1)

function hash_screen()
 local h=0
 for i=0,8191 do
  h=(h+peek(0x6000+i)*(i+1))%99999999
 end
 return h
end

-- case 3: no fillp at all (baseline) -- run FIRST, before any fillp/color
-- register writes, so no cross-case state pollution can taint it
cls()
spr(0,0,0)
printh("RESULT spr_no_fillp "..hash_screen())

-- case 1: plain checkerboard fillp active during spr(), no apply bit set
cls()
fillp(0b0011001111001100)
spr(0,0,0)
fillp()
color(6)
printh("RESULT spr_plain_fillp "..hash_screen())

-- case 2: fillp with the sprite-apply bit (0x1000) set, secondary color 0xe
cls()
fillp(0b0011001111001100 | 0x1000)
color(0x1e)
spr(0,0,0)
fillp()
color(6)
printh("RESULT spr_apply_bit_fillp "..hash_screen())

-- case 4: fillp active during sspr()
cls()
fillp(0b0011001111001100)
sspr(0,0,8,8,0,0,16,16)
fillp()
printh("RESULT sspr_plain_fillp "..hash_screen())

-- case 5: fillp active during map()
cls()
fillp(0b0011001111001100)
map(0,0,0,0,2,1)
fillp()
printh("RESULT map_plain_fillp "..hash_screen())

-- case 6: fillp active during tline()
cls()
fillp(0b0011001111001100)
tline(0,64,120,64,0,0,0.125,0)
fillp()
printh("RESULT tline_plain_fillp "..hash_screen())

printh("DONE")
