#!/usr/bin/env python3
"""Inject the per-frame screen-hash shim into a .p8 cart.

Usage: instrument.py <in.p8> <out.p8> <frames>

The shim has to work identically on the official binary and on pico-r, which
disagree about how frames are driven, so it hooks BOTH ends:

  * `flip` is wrapped by a PREPENDED block. Official's main loop calls the
    Lua-global `flip` after `_draw` (oracle-confirmed, looked up fresh each
    frame), and old-style carts with a top-level `while true do ... flip() end`
    loop call it themselves -- so this one hook covers both cart styles there.
    pico-r's main loop does NOT call the global `flip` (a real divergence, on
    record), so on pico-r this only fires for the top-level-loop carts.
  * `_draw` is wrapped by an APPENDED block, which is what covers the
    `_draw`-style carts on pico-r. It has to be appended rather than prepended
    because `_draw` does not exist yet when the prepended block runs.

A flag keeps that from double-counting where both fire: the `_draw` wrapper
records that it dumped, and the `flip` wrapper skips (and clears) on that.

The srand pin MUST be prepended, not appended: official seeds its RNG from
wall-clock time at boot, so an unpinned cart is not reproducible against even
itself, and any srand the cart does at load time has to be overridden before
it runs. `srand` is then stubbed out so the cart cannot re-randomise later.
Note this is a real behaviour change applied equally to both engines -- it
trades away the cart's own seeding to buy determinism.

btn/btnp are pinned to false because the two engines disagree about resting
input state (pico-r's run-cart injects synthetic presses on early frames).
That means this harness only ever exercises each cart's no-input path.
"""

import sys

PREPEND = """srand(1)srand=function()end
__hf=0 __hd=false
function __hdump()
 __hf+=1
 local h,s,nz=0,0,0
 for a=0x6000,0x7ffc,4 do
  local v=peek4(a)
  h=bxor(rotl(h,7),v) s+=v
  if (v!=0) nz+=1
 end
 printh("HF "..__hf.." "..tostr(h,1).." "..tostr(s,1).." "..nz)
 if (__hf>=%d) extcmd("shutdown")
end
btn=function() return false end
btnp=function() return false end
do local _f=flip
 flip=function()
  if (not __hd) __hdump()
  __hd=false
  return _f()
 end
end
"""

APPEND = """
if _draw then
 local _od=_draw
 _draw=function() _od() __hdump() __hd=true end
end
"""


def main():
    src, dst, frames = sys.argv[1], sys.argv[2], int(sys.argv[3])
    text = open(src, encoding="latin-1").read()

    lua_tag = "__lua__\n"
    i = text.find(lua_tag)
    if i < 0:
        raise SystemExit(f"{src}: no __lua__ section")
    head = text[: i + len(lua_tag)]
    rest = text[i + len(lua_tag) :]

    # The Lua section ends at the next __<name>__ data section, if any.
    end = len(rest)
    for line_start in range(len(rest)):
        if rest.startswith("__", line_start) and (
            line_start == 0 or rest[line_start - 1] == "\n"
        ):
            nl = rest.find("\n", line_start)
            tag = rest[line_start : nl if nl >= 0 else len(rest)]
            if tag.endswith("__") and len(tag) > 4:
                end = line_start
                break
    lua, tail = rest[:end], rest[end:]

    out = head + (PREPEND % frames) + lua + APPEND + "\n" + tail
    open(dst, "w", encoding="latin-1").write(out)


if __name__ == "__main__":
    main()
