use pico_r::{
    cart,
    memory::{self, Memory},
    preprocessor,
    state::PicoState,
};

#[test]
fn p8_text_cart_loads_lua_and_data_sections() {
    let data = b"pico-8 cartridge // http://www.pico-8.com\n\
version 41\n\
__lua__\n\
x=1\n\
__gfx__\n\
0123456789abcdef\n\
__gff__\n\
ff00\n\
__map__\n\
2a00\n\
__sfx__\n\
00010203\n\
__music__\n\
01000000000\n";

    let mut state = PicoState::new();
    let cart = cart::load_bytes(data, &mut state.memory).expect("load text cart");

    assert_eq!(cart.lua_code, b"x=1\n");
    for x in 0..16 {
        assert_eq!(state.memory.sprite_get(x, 0), x);
    }
    assert_eq!(state.memory.ram[memory::ADDR_FLAGS as usize], 0xff);
    assert_eq!(state.memory.ram[memory::ADDR_FLAGS as usize + 1], 0x00);
    assert_eq!(state.memory.map_get(0, 0), 0x2a);
    assert_eq!(state.memory.ram[memory::ADDR_SFX as usize], 0x00);
    assert_eq!(state.memory.ram[memory::ADDR_SFX as usize + 1], 0x01);
    assert_eq!(state.memory.ram[memory::ADDR_SFX as usize + 2], 0x02);
    assert_eq!(state.memory.ram[memory::ADDR_SFX as usize + 3], 0x03);
    assert_eq!(state.memory.ram[memory::ADDR_MUSIC as usize] & 0x80, 0x80);
}

#[test]
fn preprocessor_keeps_pico8_dialect_transforms_stable() {
    assert_eq!(preprocessor::preprocess(b"a!=b"), b"a~=b");
    assert_eq!(preprocessor::preprocess(b"?a"), b"print(a)");
    assert_eq!(preprocessor::preprocess(b"x=0b101.1"), b"x=5.5");
}

#[test]
fn preprocessor_maps_high_byte_glyphs_to_p8_identifiers() {
    // 0x80 is a P8SCII glyph that's not a button shorthand — it passes
    // through raw (the lexer treats high bytes as identifier characters,
    // so a glyph variable's real name IS its raw byte, matching official
    // where `t={<glyph>=1}` and `t["<glyph>"]` are the same slot).
    let src: &[u8] = b"x=\x80";
    let out = preprocessor::preprocess(src);
    assert!(
        out.windows(1).any(|w| w == [0x80]),
        "expected raw glyph byte to pass through; got {out:?}"
    );
}

#[test]
fn preprocessor_skips_short_if_when_body_continues_with_or_and() {
    // Short-if expansion must NOT fire when the body looks like a multi-line
    // condition continuation. Otherwise `if (a) or` becomes `if a then or end`.
    let src: &[u8] = b"if (a) or\n(b) then end";
    let out = preprocessor::preprocess(src);
    let out = String::from_utf8_lossy(&out);
    assert!(
        !out.contains("then or"),
        "short-if should not fire on or-continuation; got {out:?}"
    );
}

#[test]
fn fixture_hello_p8_loads_clean() {
    let data = include_bytes!("fixtures/hello.p8");
    let mut state = PicoState::new();
    let cart = cart::load_bytes(data, &mut state.memory).expect("hello.p8 loads");
    assert!(String::from_utf8_lossy(&cart.lua_code).contains("function _init"));
}

#[test]
fn memory_page_registers_redirect_screen_and_sprite_access() {
    let mut memory = Memory::new();
    memory.init_draw_state();

    memory.screen_set(2, 3, 12);
    let screen_byte = memory.ram[memory::ADDR_SCREEN as usize + 3 * 64 + 1];
    assert_eq!(screen_byte & 0x0f, 12);

    memory.ram[memory::ADDR_SCREEN_PAGE as usize] = 0x00;
    memory.screen_set(0, 0, 7);
    assert_eq!(memory.ram[0] & 0x0f, 7);

    memory.ram[memory::ADDR_SCREEN as usize] = 0x0b;
    memory.ram[memory::ADDR_SPRITE_PAGE as usize] = 0x60;
    assert_eq!(memory.sprite_get(0, 0), 0x0b);
}
