pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- probe: does the fill-shape color argument's high bits (above bit 7),
-- specifically the invert-fill flag documented as 0x1800.xxxx layered on
-- top of the low-byte color index, actually invert circfill/rectfill/
-- ovalfill when 0x5f34 bit 1 (0x2) is set to gate the feature?
-- also checks whether get_color()-style truncation silently drops the
-- flag when the gate bit is OFF (should be a no-op either way), and
-- whether rectfill/ovalfill have any invert behavior at all.

function hash_screen()
 local h=0
 for i=0,8191 do
  h=(h+peek(0x6000+i)*(i+1))%99999999
 end
 return h
end

-- case 1: circfill, gate ON, high invert bits set on color arg
poke(0x5f34,0x2)
cls()
circfill(64,64,20,0x1800+7)
printh("RESULT circfill_invert_gate_on "..hash_screen())

-- case 2: circfill, gate ON, NO high bits on color arg (plain color 7) --
-- baseline to diff against case 1
poke(0x5f34,0x2)
cls()
circfill(64,64,20,7)
printh("RESULT circfill_plain_gate_on "..hash_screen())

-- case 3: circfill, gate OFF, high invert bits set on color arg -- should
-- equal a plain fill with color 7 if high bits are correctly masked off
poke(0x5f34,0x0)
cls()
circfill(64,64,20,0x1800+7)
printh("RESULT circfill_invert_gate_off "..hash_screen())

-- case 4: rectfill, gate ON, high invert bits set
poke(0x5f34,0x2)
cls()
rectfill(20,20,90,90,0x1800+8)
printh("RESULT rectfill_invert_gate_on "..hash_screen())

-- case 5: rectfill, gate ON, plain color 8 -- baseline vs case 4
poke(0x5f34,0x2)
cls()
rectfill(20,20,90,90,8)
printh("RESULT rectfill_plain_gate_on "..hash_screen())

-- case 6: ovalfill, gate ON, high invert bits set
poke(0x5f34,0x2)
cls()
ovalfill(10,10,100,90,0x1800+9)
printh("RESULT ovalfill_invert_gate_on "..hash_screen())

-- case 7: ovalfill, gate ON, plain color 9 -- baseline vs case 6
poke(0x5f34,0x2)
cls()
ovalfill(10,10,100,90,9)
printh("RESULT ovalfill_plain_gate_on "..hash_screen())

printh("DONE")
