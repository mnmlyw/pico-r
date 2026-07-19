pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
srand(1)

-- reload(dst,src,len) with NO filename is a plain self-cart rom->ram
-- copy; it never takes the external-file branch, so it's a clean,
-- filename-independent probe of "what is currently in this cart's own
-- rom at address <addr>". Use it as a side-channel to check whether a
-- cstore(...,"file") call ever touched this cart's own rom.
local function ownrom(addr)
  poke(0,0)
  reload(0,addr,1)
  return peek(0)
end

printh("RESULT rom0_before_any_cstore "..ownrom(0))

-- write a marker into ram, then cstore it to a DIFFERENT, named external
-- cart. correct semantics: this must land only in that external cart's
-- rom, leaving the current cart's own rom untouched.
poke(0,199)
cstore(0,0,1,"pico_r_probe_ext.p8")
printh("RESULT rom0_after_cstore_with_filename "..ownrom(0))

-- second marker/address pair, to make sure the result isn't a fluke of
-- address 0 specifically
poke(4,77)
cstore(4,4,1,"pico_r_probe_ext.p8")
printh("RESULT rom4_after_cstore_with_filename "..ownrom(4))

-- sanity: a plain cstore with NO filename is documented + already known
-- to write straight through to this cart's own rom, so it should change
-- rom0 here regardless of the bug under test.
poke(0,222)
cstore(0,0,1)
printh("RESULT rom0_after_cstore_no_filename "..ownrom(0))

printh("DONE")
