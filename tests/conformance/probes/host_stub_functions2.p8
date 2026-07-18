pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- `load(filename)` (multi-cart "warp" switch) and `tline(...)` (textured
-- line) were both completely unimplemented, crashing any cart that calls
-- them with "attempt to call a nil value" -- confirmed both are real
-- PICO-8 API functions via oracle (neither errors; `load` targeting a
-- nonexistent cart just lets execution continue past it), stubbed as
-- no-ops like the existing menuitem/extcmd/flip/_set_fps/serial host
-- integration points. Confirmed real-world impact (solitomb-2.p8.png:
-- `load'solitomb_title'`; tomorrow-6.p8.png: `tline(...)`).
load("nonexistent_cart")
tline(0,64,127,64,0,0,1,0)
printh("RESULT stubs_dont_crash ok")
printh("DONE")
