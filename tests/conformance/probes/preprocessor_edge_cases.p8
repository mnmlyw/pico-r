pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- 1. Compound-assign RHS extraction must not stop right after and/or --
-- a very common PICO-8 idiom, `x+=cond and a or b`, was splitting into
-- `x = x + (cond and) a or b`, leaving the right operand dangling.
local axis="x"
local rem=0
rem+=axis=="x" and 5 or 9
printh("RESULT compound_and_or "..rem)

-- 2. A short-if's dangling-operator continuation (condition spanning
-- two physical lines via a trailing binary operator, e.g. `==`) must
-- not be misexpanded as if the operator fragment were the if-body.
local a=1
local b=2
if(a>0)==
(b>0) then
  printh("RESULT dangling_op_cond matched")
else
  printh("RESULT dangling_op_cond nomatch")
end

-- 3. A short-if whose body is a `?` print-shorthand must not have its
-- synthetic "end" swallowed by the later ?-conversion pass (which also
-- consumes to end-of-line) -- `if(cond)?args` must become
-- `if cond then print(args) end`, not `if cond then print(args end)`.
local msg="hi"
if(msg~=nil)?msg,0,0,8
printh("RESULT shortif_question_mark ok")

printh("DONE")
