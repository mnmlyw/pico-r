pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- probe (load-fail case, EMPTY golden): first end closes short-if AND the enclosing function; 'return 9' is
-- then a legal top-level return and the final end is dangling.
-- Official PICO-8 fails this cart at load (verified directly; it
-- prints a syntax error and produces no RESULT output), so the
-- golden is empty -- pico-r conforms by ALSO failing to load it
-- (exact error text is not compared, only the absence of output).
function g(v) if(v==1)return 5 end return 9 end
printh("RESULT g1 "..tostr(g(1)))
printh("RESULT g2 "..tostr(g(2)))
printh("DONE")
