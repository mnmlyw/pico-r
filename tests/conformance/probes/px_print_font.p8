pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- pixel probe: system font, case-flip, wrapping, control chars
cls()
print("abcdefghijklmnopqrstuvwxyz",0,0,7)
print("ABCDEFGHIJKLMNOPQRSTUVWXYZ",0,8,8)
print("0123456789 !\"#$%&'()*+,-./",0,16,9)
print(":;<=>?@[\\]^_`{|}~",0,24,10)
-- newline + auto-cursor chaining
print("one\nrtwo",0,32,11)
print("chain1")
print("chain2")
-- negative / fractional coords
print("neg",-3,60.5,12)
-- tabs and repeated char
print("a\tb\tc",0,70,13)
print("\*5xy",0,78,14)
-- wide P8SCII glyph and hiragana
print("\x83\x86\x88",0,86,7)
print("x1",100,86,8)
print("^caret",60,60,15)
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
