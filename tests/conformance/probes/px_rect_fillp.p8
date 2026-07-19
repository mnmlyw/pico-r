pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- pixel probe: rect/rectfill incl. reversed coords, fillp patterns
cls()
rect(2,2,30,20,7)
rectfill(34,2,60,20,8)
rect(90,20,64,2,9)
rectfill(120,20,100,2,10)
rect(5,5,5,5,11)
rectfill(15,10,15,15,12)
-- fillp basic checkerboard on rect + circle
fillp(0b0101101001011010)
rectfill(2,26,40,50,7)
circfill(60,38,10,8)
-- two-color fill (high nibble)
rectfill(76,26,120,50,0x4e)
-- transparency bit
fillp(0b0101101001011010.1)
rectfill(2,56,40,80,9)
rectfill(20,60,60,76,0xa3)
fillp()
rectfill(70,56,90,70,13)
-- fillp affects line/oval?
fillp(0b0011001111001100)
line(2,86,60,86,7)
line(2,88,60,98,8)
oval(70,84,110,100,9)
ovalfill(112,84,126,100,10)
fillp()
-- pset ignores fillp?
fillp(0b1111111111111110)
for i=0,10 do pset(2+i,104,7) end
fillp()
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
