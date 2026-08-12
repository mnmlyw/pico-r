pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- probe (load-fail case, EMPTY golden): the block-if (if(cond)do) does not auto-close at end-of-line; line
-- 2's end closes it, and the for is never closed.
-- Official PICO-8 fails this cart at load (verified directly; it
-- prints a syntax error and produces no RESULT output), so the
-- golden is empty -- pico-r conforms by ALSO failing to load it
-- (exact error text is not compared, only the absence of output).
x=0 for i=1,3 do if(i==2)do do x+=1 end x+=10
end x+=100
printh("RESULT x "..x)
printh("DONE")
