pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- Official PICO-8 has no pcall, no error(), and no string
-- table/colon-methods at all -- confirmed via oracle across cart-version
-- headers 8-42 and cross-referenced against the manual.
printh("RESULT has_pcall "..tostr(pcall))
printh("RESULT has_error "..tostr(error))
printh("RESULT has_string "..tostr(string))
printh("RESULT has_assert "..tostr(assert))
printh("RESULT has_setmetatable "..tostr(setmetatable))
printh("DONE")
