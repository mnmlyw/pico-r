pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
printh("RESULT count_str "..tostr(count("8")))
printh("RESULT count_str2 "..tostr(count("abc")))
printh("RESULT count_tbl "..tostr(count({1,2})))
printh("RESULT count_empty "..tostr(count({})))
printh("RESULT count_nil "..tostr(count(nil)))
printh("DONE")
