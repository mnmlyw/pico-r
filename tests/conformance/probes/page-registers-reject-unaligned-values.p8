pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- 0x5F54 (sprite page) and 0x5F55 (screen page) redirect where sprite reads
-- and screen writes land. They are NOT a free "base = byte<<8": the console
-- validates the byte and silently ignores anything it does not like, leaving
-- the base at its default.
--
-- Swept 0..255: the only accepted pages are 8 KB-aligned, are not the map
-- (0x2000) or SFX (0x4000) region, and leave 8 KB of RAM above them --
-- {0x00, 0x60, 0x80, 0xA0, 0xC0, 0xE0}. Note 0x20 and 0x40 ARE 8 KB-aligned
-- and are still rejected, so alignment alone is not the rule.
--
-- Real carts depend on the rejection: gunturtle_cafe-0 leaves 0x5F55 set to 1
-- every frame and relies on the console ignoring it.
for _,v in pairs({0x00,0x01,0x20,0x40,0x5f,0x60,0x61,0x7f,0x80,0xa0,0xb0,0xc0,0xe0,0xf0,0xff}) do
	poke(0x6000,0) poke(v*256,0)
	poke(0x5f55,v)
	pset(0,0,7)
	poke(0x5f55,0x60)
	-- honoured => the pixel landed at v*256; ignored => it landed at 0x6000
	printh("RESULT screenpage_"..v.." at_page="..peek(v*256).." at_default="..peek(0x6000))
end

-- the register still READS BACK whatever was poked, even when ignored
poke(0x5f55,0x37)
printh("RESULT rejected_still_readable "..peek(0x5f55))
poke(0x5f55,0x60)

-- 0x5F54 has the same accepted set, checked through a sprite read
for _,v in pairs({0x00,0x01,0x20,0x60,0x80,0xff}) do
	poke(0x0000,0) poke(v*256,0x0e)
	poke(0x5f54,v)
	printh("RESULT spritepage_"..v.." sget="..sget(0,0))
	poke(0x5f54,0x00)
end

printh("DONE")
