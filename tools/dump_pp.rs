use pico_r::{cart, preprocessor, state::PicoState};
fn main() {
    let data = std::fs::read("/tmp/celeste2.p8.png").unwrap();
    let mut state = PicoState::new();
    let cart = cart::load_bytes(&data, &mut state.memory).unwrap();
    let p = preprocessor::preprocess(&cart.lua_code);
    std::fs::write("/tmp/celeste2_preprocessed.lua", &p).unwrap();
    eprintln!("wrote {} bytes", p.len());
}
