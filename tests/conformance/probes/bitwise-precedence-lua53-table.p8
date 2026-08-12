pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- The full binding table for every infix bitwise/shift operator, against
-- arithmetic, concat, each other, and comparison.
--
-- The preprocessor rewrites `a & b` into `band(a,b)` as TEXT, so it has to
-- decide each operand's extent without a real parser. Getting that wrong is
-- invisible: the cart still loads and still exits 0, it just computes
-- different numbers. This probe pins the actual binding order so the
-- rewriter can be checked against it.
--
-- Confirmed order, tightest to loosest -- exactly Lua 5.3, NOT C
-- (in C, bitwise binds looser than comparison; here it binds tighter):
--   ^ / unary / * / % \ / + - / .. / << >> >>> <<> >>< / & / ^^ ~ / | /
--   comparison / and / or
function r(n,v) printh("RESULT "..n.." "..tostr(v)) end

-- arithmetic binds TIGHTER than any bitwise op
r("add_and",   1+2&3+4)
r("add_shr",   6+4>>1)
r("mod_shl",   7%4<<2)
r("mul_and",   2*3&4)
r("neg_and",   -1&3)

-- among the bitwise ops: shifts tighter than &, & tighter than ^^/~,
-- ^^/~ tighter than |; same-level runs are left-associative
r("or_and",    4|3&1)
r("or_xor",    3|4^^5)
r("xor_and",   3^^4&5)
r("and_or_mix",12&10|5)
r("shl_and",   1<<2&4)
r("shl_shr",   8>>1<<2)
r("and_and",   7&6&5)
r("tilde_and", 3~4&5)
r("or_tilde",  4|3~1)
r("rotl_and",  1<<>1&2)

-- concat binds TIGHTER than every bitwise op (so the left operand of `&`
-- here is the whole `1 .. 2`, and the right operand of `<<` is `1 .. 2`)
r("cat_and",   1 .. 2 & 3)
r("and_cat",   1&3 .. 5)
r("shl_cat",   1<<1 .. 2)
r("or_cat",    1 | 2 .. 3)

-- whitespace is not a boundary: the right operand keeps going through it
r("ws_lshr",   1 >>> 16 - 8)
r("ws_shr",    64 >> 2 + 1)

-- comparison binds LOOSER, so the bitwise op resolves first
r("and_lt",    2&3<1)
r("and_gt",    3&1>0)
r("shl_gt",    1<<2>2)

-- the shape real carts actually use for bitmask flag tests
o=36 e=2
r("corpus_flag",  o&1<<e>0)
r("corpus_flag2", o&1<<16-14~=0)

printh("DONE")
