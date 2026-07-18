pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- extract_lhs's backward scan (used to find a compound-assign's target,
-- e.g. `x-=1`) walked back through a `}`-closed table constructor, or a
-- quote-closed string, that actually belonged to a completely different,
-- already-finished PRIOR statement -- splicing the two statements
-- together into one bogus LHS. `}` (table constructors are never a valid
-- assignment target at all) is now never consumed by this scan; a closing
-- quote is only treated as call-sugar (`f"str"`) when an identifier
-- genuinely precedes its matching open quote. Confirmed against real
-- corpus carts (samurise-1.p8.png: `local d=rnd{-1,1}n-=d*cos(e)`;
-- blood_of_vladula-0.p8.png: `local n="0123456789abcdef"b..=sub(...)`).
local function pick(t) return t[1] end
local n=5 local d=pick{7}n-=d*2
printh("RESULT brace_prefix_compound "..n)

local b="start"local x="0123456789abcdef"b..="tail"
printh("RESULT quote_prefix_compound "..b)

printh("DONE")
