pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- probe (load-fail case, EMPTY golden): the end on line 2 already cascade-closed the for opened on line 1,
-- so the standalone end on a later line is dangling.
-- Official PICO-8 fails this cart at load (verified directly; it
-- prints a syntax error and produces no RESULT output), so the
-- golden is empty -- pico-r conforms by ALSO failing to load it
-- (exact error text is not compared, only the absence of output).
x=0 for i=1,3 do
if(i==2)x+=1 end x+=10
x+=100
end
x+=1000
printh("RESULT x "..x)
printh("DONE")
