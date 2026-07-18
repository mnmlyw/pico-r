pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- `_ENV` was completely unimplemented -- resolving to `nil` -- so any cart
-- using the real Lua 5.2 `_ENV[name] = value` idiom to define globals
-- dynamically (a shared "class helper" snippet copied across several corpus
-- carts) crashed with "attempt to index a nil value". Confirmed against
-- official PICO-8 that `_ENV` is a distinct table identity from `_G`
-- (`_ENV == _G` is false, unlike stock Lua 5.2's top-level chunk), but
-- writes/reads through `_ENV[k]` still observably affect real globals.
_ENV["foo"]=42
printh("RESULT foo "..foo)
_ENV.bar=7
printh("RESULT bar "..bar)
printh("RESULT env_is_g "..tostr(_ENV==_G))
printh("DONE")
