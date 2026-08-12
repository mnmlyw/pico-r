pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- probe (load-fail case, EMPTY golden): short-if at top level followed by end: never absorbed, always a
-- syntax error when no block is open.
-- Official PICO-8 fails this cart at load (verified directly; it
-- prints a syntax error and produces no RESULT output), so the
-- golden is empty -- pico-r conforms by ALSO failing to load it
-- (exact error text is not compared, only the absence of output).
x=0 if(x==0)x+=1 end
printh("RESULT x "..x)
printh("DONE")
