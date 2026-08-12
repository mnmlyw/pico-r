pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- probe (load-fail case, EMPTY golden): two nested short-ifs then TWO ends: the first end flushes BOTH
-- synthetics and closes the for, so the second end is dangling.
-- Official PICO-8 fails this cart at load (verified directly; it
-- prints a syntax error and produces no RESULT output), so the
-- golden is empty -- pico-r conforms by ALSO failing to load it
-- (exact error text is not compared, only the absence of output).
x=0 for f in all({{v=1},{v=2},{v=3}}) do if(f.v>1)if(f.v==2)x+=1 end end x+=100
printh("RESULT x "..x)
printh("DONE")
