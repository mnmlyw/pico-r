pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- Empirically discover the sprite/map shared-memory address formula.
-- Poke distinguishable bytes at 0x1000.. and read back via mget at
-- various map rows/cols, and via sget at various sprite rows/cols.
for k=0,7 do poke(0x1000+k, 100+k) end
for x=0,7 do
  printh("RESULT mget_row32_x"..x.." "..mget(x,32))
end
for x=0,7 do
  printh("RESULT mget_row33_x"..x.." "..mget(x,33))
end
-- also check row 63/64 boundary and row 31 (should be unaffected, own region)
printh("RESULT mget_row31_x0 "..mget(0,31))

-- now check via sget: which sprite (x,y) sees the poked bytes?
for y=62,66 do
  for x=0,3 do
    local v=sget(x,y)
    if v~=0 then
      printh("RESULT sget_y"..y.."_x"..x.." "..v)
    end
  end
end
printh("DONE")
