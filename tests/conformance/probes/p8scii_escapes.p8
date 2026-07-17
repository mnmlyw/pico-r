pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
function dump(name,s)
  local o=""
  for i=1,#s do o=o..ord(sub(s,i,i)).." " end
  printh("RESULT "..name.." len="..#s.." bytes="..o)
end
dump("caret_w","\^w")
dump("caret_t","\^t")
dump("caret_bang6","\^!123456")
dump("hash_0","\#0")
dump("hash_1","\#1")
dump("dash","\-5")
dump("plus","\+5")
dump("star","\*3a")
dump("pipe","\|")
dump("caret_colon","\^:0123456789abcdef")
printh("DONE")
