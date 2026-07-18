pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- all() over a STRING iterates its characters as 1-char strings
-- (oracle-confirmed) -- samurise-1.p8.png's embedded parens8 LISP VM
-- tokenizes with `for e in all(" \n\t")`, and an empty iterator here
-- spun its scanner loop forever.
local acc=""
for c in all("abc") do acc=acc..c.."," end
printh("RESULT all_str "..acc)
local n=0
for c in all("") do n+=1 end
printh("RESULT all_empty "..n)
printh("RESULT s0 "..tostr(("xy")[0]))
printh("DONE")
