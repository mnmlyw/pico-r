pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- in-cart model of pow using sqrt(), *, /, compare against ^
function powm(x,y)
 if y<0 then x=1/x y=-y end
 local n=flr(y)
 local f=y-n
 local r=1
 local b=x
 while n>0 do
  if n%2>=1 then r=r*b end
  b=b*b
  n=flr(n/2)
 end
 if f~=0 then
  if x<=0 then return 0 end
  local a=x
  local bit=0.5
  while f~=0 and bit~=0 do
   a=sqrt(a)
   if f>=bit then r=r*a f=f-bit end
   bit=bit/2
  end
 end
 return r
end

local mism=0
local tested=0
function chk(x,y)
 tested+=1
 local p=x^y
 local m=powm(x,y)
 if p~=m then
  mism+=1
  if mism<=40 then
   printh("RESULT MISM "..tostr(x,0x2).." "..tostr(y,0x2).." "..tostr(p,0x2).." "..tostr(m,0x2))
  end
 end
end

-- deterministic random sweep
srand(7)
for i=1,300 do
 local x=rnd(20)
 local y=rnd(8)-4
 chk(x,y)
end
for i=1,200 do
 local x=rnd(2)
 local y=rnd(30)-15
 chk(x,y)
end
for i=1,100 do
 local x=rnd(32000)
 local y=rnd(4)-2
 chk(x,y)
end
for i=1,100 do
 local x=rnd(0x0.01)
 local y=rnd(6)-3
 chk(x,y)
end
-- negatives and specials
chk(-2,2.5) chk(-2,-2.5) chk(-2,3) chk(-2,-3)
chk(0,0) chk(0,2) chk(0,-2) chk(0,0.5) chk(0,-0.5)
for i=1,100 do
 local x=rnd(40)-20
 local y=rnd(20)-10
 chk(x,y)
end
printh("RESULT SUMMARY "..tested.." "..mism)
-- y=-32768 edge
printh("RESULT edge_2 "..tostr(2^0x8000,0x2))
printh("RESULT edge_h "..tostr(0.5^0x8000,0x2))
printh("RESULT edge_1 "..tostr(1^0x8000,0x2))
printh("RESULT edge_m "..tostr(powm(2,0x8000),0x2))
-- sqrt map for algorithm derivation
for k=1,120 do
 local x=k*0x10.123+0x0.777
 printh("RESULT sq "..tostr(x,0x2).." "..tostr(sqrt(x),0x2))
end
for k=1,60 do
 local x=k*0x0.0123
 printh("RESULT sq "..tostr(x,0x2).." "..tostr(sqrt(x),0x2))
end
printh("RESULT sq "..tostr(0x7fff.ffff,0x2).." "..tostr(sqrt(0x7fff.ffff),0x2))
printh("RESULT sq_neg "..tostr(sqrt(-4),0x2))
printh("RESULT sqmax2 "..tostr(sqrt(0x7fff.0000),0x2))
-- perfect squares and near-boundary values
for k=1,20 do
 printh("RESULT sq "..tostr(k*k,0x2).." "..tostr(sqrt(k*k),0x2))
end
printh("DONE")
