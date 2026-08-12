pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- probe: PICO-8's short-if/short-while "deferred end" balance rules,
-- consolidated from ~45 individually-oracle-verified single-case carts
-- (see LEDGER). The rule: a short form's body runs to end-of-line OR to
-- the first `end`/`until` not matched by a statement opener (if/while/
-- for/function/repeat/bare-do) inside the body; that token flushes the
-- synthetic `end`s of ALL pending short forms immediately before it and
-- is left in place to close the next enclosing block -- even one opened
-- on an earlier line. `else`/`elseif` absorb into the short-if.
-- `if(cond)do` is a block-if (`do` acts as the separator), not a short
-- form. Each section below re-initializes every variable it reads, so
-- the cases compose; RESULT lines keep their original per-case labels
-- and are compared as an ordered sequence.

-- [fable/case1]
x=0 for f in all({{v=1},{v=2},{v=3}}) do if(f.v==2)x+=1 end x+=100
printh("RESULT x "..x)
-- [fable/case1s]
x=0 for f in all({{v=1},{v=2},{v=3}}) do if (f.v==2) x+=1 end x+=100
printh("RESULT x "..x)
-- [fable/case2]
x=0 for f in all({{v=1},{v=2},{v=3}}) do if(f.v==2)x+=1 x+=10 end x+=100
printh("RESULT x "..x)
-- [fable/case3]
x=0 n=0 while n<3 do n+=1 if(n==2)x+=1 end x+=100
printh("RESULT x "..x)
-- [fable/case4]
x=0 n=0 for i=1,3 do while(x<2)x+=1 end n+=100
printh("RESULT x "..x.." n "..n)
-- [fable/case5]
x=0 for f in all({{v=1},{v=2},{v=3}}) do if(f.v==2)x+=1 end
x+=100
printh("RESULT x "..x)
-- [fable/case6]
x=0 y=0 for f in all({{v=1},{v=2},{v=3}}) do if(f.v==2)x+=1 end if(x>0)y+=1
printh("RESULT x "..x.." y "..y)
-- [fable/case6s]
x=0 y=0 for f in all({{v=1},{v=2},{v=3}}) do if (f.v==2) x+=1 end if (x>0) y+=1
printh("RESULT x "..x.." y "..y)
-- [agent2/case1a]
x=0 for i=1,2 do for j=1,3 do if(j==2)x+=1 end end x+=100
printh("RESULT x "..x)
-- [agent2/case1b]
x=0 for i=1,2 do for j=1,3 do if(j==2)x+=1 end end x+=100
x+=1000
printh("RESULT x "..x)
-- [agent2/case2]
x=0 for f in all({{v=1},{v=2},{v=3}}) do if(f.v>1)if(f.v==2)x+=1 end x+=100
printh("RESULT x "..x)
-- [agent2/case4]
x=0 for f in all({{v=1},{v=2},{v=3}}) do if(f.v==2)do x+=1 end x+=10 end x+=100
printh("RESULT x "..x)
-- [agent2/case4b]
x=0 if(x>0)do x+=1 end x+=10
printh("RESULT x "..x)
-- [agent2/case4c]
x=0 if(x>0) do x+=1 end x+=10
printh("RESULT x "..x)
-- [agent2/case5]
x=0 for i=1,2 do for f in all({{v=1},{v=2},{v=3}}) do if(f.v==2)for k=1,4 do x+=1 end x+=10 end x+=100 end x+=1000
printh("RESULT x "..x)
-- [agent2/case6b]
x=0 for i=1,3 do
if(i==2)x+=1 end x+=10
x+=100
printh("RESULT x "..x)
-- [agent2/case7space]
x=0 for i=1,2 do for j=1,3 do if (j==2) x+=1 end end x+=100
printh("RESULT x "..x)
-- [agent2/case8then]
x=0 for i=1,3 do if(i==2)if i>0 then x+=1 end x+=10 end x+=100
printh("RESULT x "..x)
-- [agent2/case9func]
x=0 for i=1,3 do if(i==2)local g=function()return 1 end x+=10 end x+=100
printh("RESULT x "..x)
-- [agent2/case10arena]
x=0 for i=1,3 do if(i==2)x+=1 end if(x>=0)x+=10
printh("RESULT x "..x)
-- [endterm/c1]
x=0 if x==0 then
if(x==9)x+=1 else x+=10 end
printh("RESULT x "..x)
-- [endterm/c1s]
x=0 if x==0 then
if (x==9) x+=1 else x+=10 end
printh("RESULT x "..x)
-- [endterm/c2a]
x=0 if x==0 then if(x==9)x+=1 else x+=10 end x+=100
printh("RESULT x "..x)
-- [endterm/c3]
x=0 if x==0 then
if(x==9)x+=1 elseif x==0 then x+=10 end
printh("RESULT x "..x)
-- [endterm/c4]
x=0 repeat
x+=1 if(x==1)x+=10 until x>5
printh("RESULT x "..x)
-- [endterm/c5]
x=0 for i=1,3 do if(i==2)g=function() return 7 end x+=10 end x+=100
printh("RESULT x "..x)
printh("RESULT g "..g())
-- [endterm/c6a]
x=0 friend=0 for i=1,3 do if(i==2)x+=1 friend+=1 end x+=100
printh("RESULT x "..x.." friend "..friend)
-- [endterm/c6b]
x=0 for i=1,3 do if(i==2)s="a end b" x+=1 end x+=100
printh("RESULT x "..x.." s "..#s)
-- [endterm/c7]
x=0 for i=1,3 do if(i==2)x+=1 else x+=10 end x+=100
printh("RESULT x "..x)
-- [endterm/c8]
x=0 for i=1,3 do if(i==2)x+=1 else x+=10 x+=1000 end x+=100
printh("RESULT x "..x)
-- [probe/case1]
x=0 for f in all({{v=1},{v=2},{v=3}}) do if(f.v==2)x+=1 end ?x
printh("RESULT x "..x)
-- [probe/case1c]
x=0 c=0 for f in all({{v=1},{v=2},{v=3}}) do if(f.v==2)x+=1 end c+=1
printh("RESULT x "..x)
printh("RESULT c "..c)
-- [probe/case2]
x=0 for i=1,3 do if(i==2)x+=1 end x+=100 --note
printh("RESULT x "..x)
-- [probe/case2b]
x=0 for i=1,3 do if (i==2) x+=1 end x+=100
printh("RESULT x "..x)
-- [probe/case3]
x=0 for i=1,3 do if(i==2)end x+=10
printh("RESULT x "..x)
-- [probe/case4]
x=0 for i=1,3 do if(i==2)x+=1 if(i==3)x+=10 end x+=100
printh("RESULT x "..x)
-- [probe/case5]
f=61 d=61 x=1 i={} add(i,{e=2,f=10,d=10,o=1,i=0}) n=i[1] for f in all(i) do if(f.e==4 and (n.f-f.f)^2<25)del(i,f) end if((n.f-f)^2+(n.d-d)^2<25)x-=.1
printh("RESULT x "..tostr(x,0x2))
-- [probe/case5c]
f=0 d=0 x=1 i={} add(i,{e=2,f=10,d=10}) add(i,{e=2,f=11,d=11}) n=i[1] for f in all(i) do if(f.e==4)del(i,f) end if((n.f-f)^2+(n.d-d)^2<250)x-=.1
printh("RESULT x "..tostr(x,0x2))
-- [probe/case6c]
function g(v) if(v==1)return 5 end
printh("RESULT g1 "..tostr(g(1)))
printh("RESULT g2 "..tostr(g(2)))
-- [probe/case7]
x=0 for i=1,3 do if(i==2)do x+=1 end x+=10 end x+=100
printh("RESULT x "..x)
-- [probe/case8]
x=0 for i=1,3 do if(i>=2)do if(i==3)x+=1 end x+=10 end x+=100
printh("RESULT x "..x)
-- [probe/case10]
x=0 i=0 for j=1,2 do while(i<3)i+=1 end x+=100
printh("RESULT i "..i)
printh("RESULT x "..x)
-- [probe/case12]
x=0 for i=1,3 do for j=1,3 do if(j==2)x+=1 end if(i==3)x+=10 end x+=100
printh("RESULT x "..x)
-- [discriminator: elseif inside nested regular if inside short-if body]
x=0 for i=1,3 do if(i==2)if x==0 then x+=1 elseif x==9 then x+=2 end x+=10 end x+=100
printh("RESULT x "..x)
printh("DONE")
