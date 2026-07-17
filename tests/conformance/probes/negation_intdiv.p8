pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- A leading unary minus directly before an int-div operand must bind
-- inside the divided expression (`flr(-a/b)`), not outside the rewritten
-- call (`-flr(a/b)`) -- these round differently since flr() rounds
-- toward negative infinity.
printh("RESULT neg_intdiv "..(-5\2))
printh("RESULT neg_intdiv2 "..(-7\2))
local a=5
printh("RESULT neg_var_intdiv "..(-a\2))
printh("RESULT neg_intdiv_zero "..(-5\0))
printh("DONE")
