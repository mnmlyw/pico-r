pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- 1. extract_rhs's glued-statement heuristic fired on the FIRST `=` of a
--    `==` comparison appearing later in a compound-assign's RHS (it only
--    recognized `==` scanning backward from the second char), mis-splitting
--    an and/or ternary chain with two comparisons. Real corpus cart:
--    puzzlesofthepaladin-3.p8.png (`i.dx+=i.go=="w"and-1or i.go=="e"and 1or 0`).
local go="w"
local dx=0
dx+=go=="w"and-1or go=="e"and 1or 0
printh("RESULT ternary_two_comparisons "..dx)

-- 2. extract_lhs walked back through a closing delimiter that directly
--    abuts the captured identifier (value juxtaposition from two
--    golfed-together statements), splicing the prior statement into the
--    LHS. Real corpus cart: praxis_fighter_x-2.p8.png
--    (`e.y-=rnd"18"e.a+=rnd"6"`).
local e={y=10,a=1}
e.y-=flr(rnd"1")e.a+=2
printh("RESULT juxtaposed_compounds "..e.y.." "..e.a)

printh("DONE")
