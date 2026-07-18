pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- sin()/cos()/atan2() are table-driven on real hardware (a 4096-entry
-- quarter-turn lookup for sin/cos, a 2049-entry lookup for atan2), not
-- continuous math -- see LEDGER.md. These probe exact table-boundary
-- inputs (n/4096), an off-boundary/half-step input between two table
-- buckets, and atan2 across all four quadrants plus axis-aligned and
-- 45-degree cases.
for n=0,4095,257 do
  printh("RESULT sin_"..n.." "..tostr(sin(n/4096),1))
end
printh("RESULT sin_half_0 "..tostr(sin(0.5/4096),1))
printh("RESULT sin_half_1 "..tostr(sin(1.5/4096),1))
printh("RESULT sin_half_2 "..tostr(sin(2.5/4096),1))
printh("RESULT sin_neg "..tostr(sin(-0.1),1))
printh("RESULT sin_wraps "..tostr(sin(3.3),1))
printh("RESULT cos_wraps "..tostr(cos(-2.7),1))
for n=0,4095,257 do
  printh("RESULT cos_"..n.." "..tostr(cos(n/4096),1))
end
printh("RESULT atan2_0_0 "..tostr(atan2(0,0),1))
printh("RESULT atan2_1_0 "..tostr(atan2(1,0),1))
printh("RESULT atan2_0_1 "..tostr(atan2(0,1),1))
printh("RESULT atan2_neg1_0 "..tostr(atan2(-1,0),1))
printh("RESULT atan2_0_neg1 "..tostr(atan2(0,-1),1))
printh("RESULT atan2_1_1 "..tostr(atan2(1,1),1))
printh("RESULT atan2_neg1_1 "..tostr(atan2(-1,1),1))
printh("RESULT atan2_1_neg1 "..tostr(atan2(1,-1),1))
printh("RESULT atan2_neg1_neg1 "..tostr(atan2(-1,-1),1))
printh("RESULT atan2_2_1 "..tostr(atan2(2,1),1))
printh("RESULT atan2_1_2 "..tostr(atan2(1,2),1))
printh("RESULT atan2_overflow "..tostr(atan2(1,0x8000),1))
printh("DONE")
