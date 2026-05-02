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

    assert_eq!(cart.lua_code, "x=1\n");
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
    assert_eq!(preprocessor::preprocess("a!=b"), "a~=b");
    assert_eq!(preprocessor::preprocess("?a"), "print(a)");
    assert_eq!(preprocessor::preprocess("x=0b101.1"), "x=5.5");
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
