pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- print()'s return value is the right-most x reached at any point while
-- printing (not wherever a trailing newline left the cursor), and it
-- always persists the cursor registers afterward regardless of which
-- argument form was used: cursor_x resets to this call's starting x,
-- cursor_y advances by 6 PER LINE printed (not a flat +6).
local r = print("hello\nb")
printh("RESULT print_retval "..r)

cursor(20,20)
print("ab",5,5,7)
printh("RESULT cx_after_explicit "..peek(0x5f26))
printh("RESULT cy_after_explicit "..peek(0x5f27))

cursor(20,20)
print("hello\nb")
printh("RESULT cx_multiline "..peek(0x5f26))
printh("RESULT cy_multiline "..peek(0x5f27))

printh("DONE")
