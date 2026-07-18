pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- Devkit/serial stats are BOOLEANS on official (oracle-confirmed):
-- stat(30) has-pending-devkit-key, stat(31) pending key string,
-- stat(28,code) raw scancode held, stat(120)/stat(121) stdin/serial
-- data pending. The old generic numeric-0 fallback was TRUTHY in Lua
-- and spun `while stat(30) do`-style drain loops forever
-- (terra_1cart-42.p8.png).
printh("RESULT s30 "..tostr(stat(30)))
printh("RESULT s31 ["..tostr(stat(31)).."]")
printh("RESULT s28 "..tostr(stat(28,8)))
printh("RESULT s120 "..tostr(stat(120)))
printh("RESULT s121 "..tostr(stat(121)))
printh("DONE")
