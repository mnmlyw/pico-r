pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- A short-if's "does this already have an explicit separator?" check
-- previously scanned the WHOLE rest of the line for a bare `then`/`do`
-- anywhere, rather than just the token immediately after the condition's
-- closing paren -- so a short-if whose body is itself a compound
-- statement containing its own `then` (a nested `if`) was wrongly treated
-- as "already has an explicit then", leaving the outer short-if's `then`
-- (and its synthetic `end`) never inserted. Confirmed against a real
-- corpus cart (tinyhawk-2.p8.png: `if(n.current_qp) if e.fliph and
-- e.flipv then ... end`).
local a,b=1,0
if(a==1) if b==0 then b=5 end
printh("RESULT nested_short_if "..b)
printh("DONE")
