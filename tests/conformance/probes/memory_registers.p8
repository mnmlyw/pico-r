pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- cstore() silently skips writes at/beyond 0x4300 (the compiled-code
-- region) rather than writing through or erroring -- confirmed with a
-- destination range straddling the boundary.
poke(0,42) cstore(0x4300,0,1) poke(0,0) reload(0,0x4300,1)
printh("RESULT cstore_protected "..peek(0))

poke(0,21) poke(1,22) poke(2,23) poke(3,24)
cstore(0x42fe,0,4)
poke(0,0) poke(1,0) poke(2,0) poke(3,0)
reload(0,0x42fe,4)
printh("RESULT cstore_partial_b0 "..peek(0))
printh("RESULT cstore_partial_b1 "..peek(1))
printh("RESULT cstore_partial_b2 "..peek(2))
printh("RESULT cstore_partial_b3 "..peek(3))

-- Memory addresses use flr() semantics: peek(-0.5) reads the same byte
-- as peek(-1) (both wrap to 0xffff), not address 0.
poke(0xffff,7)
printh("RESULT peek_neg_half "..peek(-0.5))
printh("RESULT peek_neg1 "..peek(-1))

printh("DONE")
