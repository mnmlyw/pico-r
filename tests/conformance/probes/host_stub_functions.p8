pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- `_set_fps(n)` (custom frame-rate override) and `serial(addr,len,channel)`
-- (host GPIO/serial data export) were both completely unimplemented,
-- crashing any cart that calls them with "attempt to call a nil value" --
-- confirmed both are real PICO-8 API functions via oracle (neither errors
-- there), stubbed as no-ops like the existing menuitem/extcmd/flip host
-- integration points. Confirmed real-world impact (bytebeat_tweet-0.p8.png:
-- `_set_fps(60)`; pico1kmatch-0.p8.png/pico1kshmup_1-0.p8.png: `serial(...)`).
_set_fps(45)
serial(2056,0,8)
printh("RESULT stubs_dont_crash ok")
printh("DONE")
