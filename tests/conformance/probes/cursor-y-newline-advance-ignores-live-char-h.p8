pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- probe: does the newline cursor_y advance use the LIVE char_h
-- (tall/pinball mode) or a hardcoded 6px/line?
cls()

-- case 1: tall mode toggled on, two newlines inside one print()
cursor(0,0)
print("\^ta\nb\nc")
printh("RESULT cy_tall "..peek(0x5f27))

-- case 2: pinball mode toggled on, one newline
cls()
cursor(0,0)
print("\^pa\nb")
printh("RESULT cy_pinball "..peek(0x5f27))

-- case 3: baseline, no mode toggles, two newlines (should be 18 either way)
cls()
cursor(0,0)
print("a\nb\nc")
printh("RESULT cy_normal "..peek(0x5f27))

-- case 4: tall mode turned back off mid-string before the newline
cls()
cursor(0,0)
print("\^ta\^-ta\nb")
printh("RESULT cy_tall_then_off "..peek(0x5f27))

printh("DONE")
