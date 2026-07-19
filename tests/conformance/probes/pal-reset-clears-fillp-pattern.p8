pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
fillp(0b0011001111001100)
printh("RESULT fillp_bytes_before_pal "..peek(0x5f31)..","..peek(0x5f32)..","..peek(0x5f33))
pal()
printh("RESULT fillp_bytes_after_pal_reset "..peek(0x5f31)..","..peek(0x5f32)..","..peek(0x5f33))
fillp(0b1010101010101010)
fillp(0,true)
printh("RESULT fillp_bytes_before_pal2 "..peek(0x5f31)..","..peek(0x5f32)..","..peek(0x5f33))
pal()
printh("RESULT fillp_bytes_after_pal_reset2 "..peek(0x5f31)..","..peek(0x5f32)..","..peek(0x5f33))
printh("DONE")
