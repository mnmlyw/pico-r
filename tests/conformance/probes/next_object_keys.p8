pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- next()/pairs() must round-trip table- and function-valued KEYS -- the
-- old registry stored only a raw pointer, so iteration returned nil for
-- object keys and ENDED EARLY, silently truncating any table keyed by
-- objects (samurise-1.p8.png's LISP VM keys upvalue sets by sentinel
-- tables and copies them with `for e in next,n do ... end`).
local sent={}
local n={}
n[sent]=true
n.plain=1
local copied=0
local sent_seen=false
for k in next,n do
 copied+=1
 if k==sent then sent_seen=true end
end
printh("RESULT count "..copied.." sent "..tostr(sent_seen))
local d={[sent]=7}
local t={}
for k in next,n do t[k]=d[k] end
printh("RESULT copy "..tostr(t[sent]))
printh("DONE")
