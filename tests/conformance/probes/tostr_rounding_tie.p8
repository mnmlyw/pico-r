pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- Confirms the round-2 "tostr's 4-decimal display doesn't handle exact-tie
-- precision" gap (LEDGER.md) auto-resolved once literals were quantized:
-- 0.12345 used to round to "0.1235" (unquantized f64), now "0.1234"
-- (matching official PICO-8, which rounds the already-quantized value).
printh("RESULT tie "..(0.12345))
printh("DONE")
