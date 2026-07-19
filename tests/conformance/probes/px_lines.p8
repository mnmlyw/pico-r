pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- pixel-conformance probe: line() in all octants, pen-state chaining,
-- fractional and off-screen endpoints. Dumps the full screen as hex.
cls()
-- all 8 octants from a hub
for i=0,15 do
 line(64,64,64+24*cos(i/16),64+24*sin(i/16),1+i%15)
end
-- pen chaining: line(x,y) continues from last endpoint
line(2,2,20,2,8)
line(30,10)
line(2,120)
-- single-arg/no-arg forms
line(40,100,50,110,10)
line()
line(60,120)
-- fractional endpoints
line(100.5,10.2,120.9,30.7,11)
line(100,30.5,120,30.49,12)
-- off-screen clamping
line(-20,50,20,58,14)
line(120,-10,140,140,13)
function dump()
 local h="0123456789abcdef"
 for r=0,127 do
  local s=""
  for b=0,63 do
   local v=peek(0x6000+r*64+b)
   s=s..sub(h,v\16+1,v\16+1)..sub(h,v%16+1,v%16+1)
  end
  printh("RESULT r"..(r<10 and "00" or (r<100 and "0" or ""))..r.." "..s)
 end
 printh("DONE")
end
dump()
