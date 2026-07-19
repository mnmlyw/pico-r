pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- pixel probe: every wide glyph 0x80-0xff, 16 per row band
cls()
for i=0,127 do
 local c=chr(128+i)
 print(c,(i%16)*8,(i\16)*8,7)
end
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
