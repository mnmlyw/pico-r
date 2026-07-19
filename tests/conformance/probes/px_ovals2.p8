pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- pixel probe: adversarial oval sizes, outline+fill pairs
cls()
oval(2,2,8,32,7)
ovalfill(12,2,18,32,8)
oval(22,2,52,8,9)
ovalfill(22,12,52,18,10)
oval(60,2,60,32,11)
oval(70,2,100,2,12)
oval(104,2,105,3,13)
oval(108,2,110,4,14)
ovalfill(114,2,117,6,15)
oval(2,40,7,49,10)
ovalfill(12,40,19,41,11)
oval(24,40,39,43,12)
ovalfill(44,40,59,44,13)
oval(64,40,67,55,14)
ovalfill(72,40,76,58,15)
oval(82,40,90,70,2)
ovalfill(94,40,102,70,1)
oval(108,40,126,45,6)
oval(2,60,33,90,5)
ovalfill(38,60,69,91,4)
oval(74,74,121,89,3)
ovalfill(74,92,121,111,13)
oval(2,96,32,126,14)
ovalfill(36,96,66,126,12)
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
