pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- 1. PICO-8 accepts C-style `//` line comments (confirmed via oracle) --
--    completely unimplemented, they reached the Lua lexer as raw slashes.
--    Real corpus carts use them (slimer2-0.p8.png, cross_battle_wip1-0.p8.png).
-- 2. A `;` must terminate a compound-assign's RHS capture -- `i+=1;return`
--    was capturing `1;` and splicing the semicolon inside the rewritten
--    parens (`i = i + (1;)`). Real corpus cart: picketpuzzle-5.p8.png.
-- 3. A `--` comment after a bitwise op's RHS was smuggled into the
--    rewritten call, eating the closing paren (`1 << 3--c` became
--    `shl(1,3--c)`). Real corpus cart: swap_mouse-8.p8.png.
local x=1 // this is a comment
// full-line comment
local y=2
x+=1;y+=2
printh("RESULT x "..x)
printh("RESULT y "..y)

local f = 1 << 3--will not swap
printh("RESULT shl_comment "..f)

printh("DONE")
