pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- probe (load-fail case, EMPTY golden): the short-if line (with absorbed else) is fully balanced by itself;
-- an explicit end on the next line is extra.
-- Official PICO-8 fails this cart at load (verified directly; it
-- prints a syntax error and produces no RESULT output), so the
-- golden is empty -- pico-r conforms by ALSO failing to load it
-- (exact error text is not compared, only the absence of output).
x=0 if x==0 then if(x==9)x+=1 else x+=10 end x+=100
end
printh("RESULT x "..x)
printh("DONE")
