pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- Two ways a short-if's synthetic closing `end` could get lost to
-- end-of-line consumers in the body:
-- 1. A trailing `--` comment: the `end` was appended after the loop,
--    landing inside the comment text. (dinkykong-0.p8.png:
--    `if(not cstg.hr)return true --todo:...`)
-- 2. A `?` print-shorthand appearing MID-body after other statements
--    (the body-START case was fixed earlier): the later ?-conversion
--    swallowed the appended `end` into print(...)'s argument list.
--    (dinkykong-0.p8.png: `if(ob.sel)rect(...) ?ob.x..","..ob.y`)
local a=1
local hit=0
if(a==1)hit=1 --trailing comment here
printh("RESULT comment_body "..hit)

local b=2
if(b==2)b=3 ?"x"
printh("RESULT midbody_qmark "..b)

-- 3. extract_lhs walking back through `..` (concat) as if it were a
--    field-access dot, splicing the concat's left side into an int-div
--    rewrite. (hakai-3.p8.png: `?"time:"..timer\1,...`)
local timer=7
printh("RESULT concat_intdiv "..("t:"..timer\2))

printh("DONE")
