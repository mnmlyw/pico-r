pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
printh("RESULT a:"..tostr(13<<-3))
printh("RESULT b:"..tostr(13>>-3))
printh("RESULT c:"..tostr(shl(13,-3)))
printh("RESULT d:"..tostr(shr(13,-3)))
printh("RESULT e:"..tostr(200<<1))
printh("RESULT f:"..tostr(0x80<<9))
printh("RESULT g:"..tostr(13<<0.5))
printh("RESULT h:"..tostr(13>>0.5))
printh("RESULT i:"..tostr((0xab<<1)%512))
printh("RESULT j:"..tostr((0xab<<-7)%512))
printh("RESULT k:"..tostr(0.5%512))
printh("RESULT l:"..tostr((0xff<<9)%512))
printh("RESULT m:"..tostr(2^9))
printh("RESULT n:"..tostr(1.125%2))
printh("RESULT o:"..tostr(-3.5%512))
printh("DONE")
