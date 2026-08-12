pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- probe (load-fail case, EMPTY golden): top-level short-if then end with nothing open: the synthetic closes
-- the if, the explicit end has nothing left to close.
-- Official PICO-8 fails this cart at load (verified directly; it
-- prints a syntax error and produces no RESULT output), so the
-- golden is empty -- pico-r conforms by ALSO failing to load it
-- (exact error text is not compared, only the absence of output).
x=0 if(x==0)x+=1 end x+=10
printh("RESULT x "..x)
printh("DONE")
