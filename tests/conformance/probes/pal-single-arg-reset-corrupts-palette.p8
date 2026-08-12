pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- pal(p) with a single integer argument: manual says it resets
-- palette p (0=draw,1=display,2=secondary) to system defaults.
-- Check whether pal(0)/pal(1) actually reset the WHOLE 16-entry
-- palette, or whether they only touch the entry indexed by the
-- argument itself (mistaking it for a scalar remap call).

-- dirty every draw-palette entry with a known remap, then reset
for c=0,15 do
	pal(c,(c+8)%16)
end
pal(0)
printh("RESULT drawpal_after_reset0 "..peek(0x5f00))
printh("RESULT drawpal9_after_reset0 "..peek(0x5f09))
printh("RESULT drawpal15_after_reset0 "..peek(0x5f0f))

-- dirty screen palette too, then reset via pal(1)
for c=0,15 do
	pal(c,(c+8)%16,1)
end
pal(1)
printh("RESULT screenpal_after_reset1 "..peek(0x5f10))
printh("RESULT screenpal9_after_reset1 "..peek(0x5f19))
printh("RESULT drawpal1_after_reset1 "..peek(0x5f01))

-- transparency bit (0x10 on draw-pal entry 0) should also come back
-- to its default (opaque, i.e. bit clear except color 0 which pico-8
-- keeps transparent by default) after a pal(0) reset
palt(0,false)
pal(0)
printh("RESULT drawpal0_transbit_after_reset0 "..peek(0x5f00))

-- pal(2) resets the secondary/"p2" palette storage at 0x5f60+ -- to a
-- fixed 16-byte system-default pattern (not derivable from a formula,
-- confirmed byte-for-byte against a dirtied-then-reset readback of all
-- 16 entries, not just one).
for k=0,15 do poke(0x5f60+k,0xff) end
pal(2)
for k=0,15 do
	printh("RESULT p2pal"..k.."_after_reset2 "..peek(0x5f60+k))
end

printh("DONE")
