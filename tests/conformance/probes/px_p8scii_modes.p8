pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- pixel probe: P8SCII rendering modes -- wide/tall/stripey/invert/border,
-- background color, vertical offsets, custom font
cls()
print("\^wab",0,0,7)
print("\^tcd",20,0,8)
print("\^w\^tef",40,0,9)
print("\^igh",70,0,10)
print("\#3ij",90,0,7)
print("\^bkl",110,0,11)
print("a\vfb\vfc",0,20,12)
print("m\-fn",30,20,13)
-- mode off switches
print("\^wop\^-wqr",50,20,14)
-- pinball: dotty mode
print("\^pst",100,20,15)
-- custom font: header at 0x5600 -- width 8, wide width 8, height 8,
-- then glyph bitmaps; enable with \014
poke(0x5600,8) poke(0x5601,8) poke(0x5602,8)
for a=0x5608,0x5fff do poke(a,(a%3==0) and 0xaa or 0x55) end
print("\14uv",0,40,7)
print("\14\^ww",30,40,8)
-- back to system font mid-string
print("\14x\15y",60,40,9)
-- one-off char size via \^x \^y? (char width override)
print("\^x9z!",90,40,10)
function dump()
 local h="0123456789abcdef"
 for r=0,127 do
  local s=""
  for b=0,63 do
   local v=peek(0x6000+r*64+b)
   s=s..sub(h,v\16+1,v\16+1)..sub(h,v%16+1,v%16+1)
  end
  printh("RESULT r"..(r<10 and "00" or (r<100 and "0" or ""))..r.." "..s)
 end
 printh("DONE")
end
dump()
