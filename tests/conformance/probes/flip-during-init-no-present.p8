pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- flip() called from top-level chunk (no _init/_update/_draw defined at
-- all here -- this *is* the "top-level chunk before _init even runs" case).
-- Probes whether flip()'s Lua-visible effects (elapsed time / frame
-- advance, and the screen buffer contents) are observable identically
-- in official PICO-8 vs pico-r when flip() is called before any game
-- loop functions exist.
srand(1)

cls(5)
flip()
cls(6)
flip()
cls(7)
flip()

-- checksum of full screen memory after the flips
local h=0
for i=0x6000,0x7fff do
 h=(h*31+peek(i))%1000000007
end
printh("RESULT screen_checksum "..tostr(h))
printh("RESULT t "..tostr(t()))
printh("RESULT time "..tostr(time()))

-- a few more flips to see if elapsed time keeps advancing linearly
flip()
flip()
printh("RESULT t2 "..tostr(t()))

printh("DONE")
