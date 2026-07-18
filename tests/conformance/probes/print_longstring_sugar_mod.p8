pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- 1. A `?`-print whose argument is a multi-line `[[ ]]` long string must
--    not close its print( paren at the first physical line's end -- the
--    close is deferred until the string (and trailing args) finish.
--    Real corpus cart: homunculus-0.p8.png (`?[[ ENTER ... ]],102,29,2`).
?[[line one
line two]],4,8

printh("RESULT multiline_print ok")

-- 2. `%` right after a string-call-sugar value is modulo, not the peek2
--    shortcut (a quote in operator position is a CLOSING quote -- a
--    string value just ended). Real corpus cart: picodex_dual-1.p8.png
--    (`T.S[p"n4"%6+1]`).
function len_of(s) return #s end
printh("RESULT sugar_mod "..(len_of"abcde"%3))

printh("DONE")
