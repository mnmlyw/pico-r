// LuaEngine abstracts over the actual Lua VM. The default `StubEngine`
// preprocesses and stores the cart source but does not execute it — useful
// for verifying the rest of the engine (cart parsing, gfx, audio, frame loop).
//
// To run real carts, swap in a piccolo-backed engine (or any pure-Rust Lua VM)
// and implement the same interface.

use crate::cart::Cart;
use crate::preprocessor;
use crate::state::PicoState;

pub trait LuaEngine {
    fn load_cart(&mut self, state: &mut PicoState, cart: &Cart) -> Result<(), String>;
    fn call_init(&mut self, state: &mut PicoState);
    fn call_update(&mut self, state: &mut PicoState);
    fn call_draw(&mut self, state: &mut PicoState);
    fn use_60fps(&self) -> bool;
    fn had_error(&self) -> bool;
    fn error_message(&self) -> &str;

    /// Serialize the Lua globals to a byte blob. Non-serializable values
    /// (functions, threads) are skipped.
    fn save_globals(&self) -> Vec<u8>;
    /// Restore globals from a previously saved blob. The cart source must
    /// already be loaded so all function definitions exist.
    fn load_globals(&mut self, data: &[u8]) -> Result<(), String>;
}

/// Stub engine — preprocesses cart code but doesn't execute it.
/// Renders a placeholder so you can confirm the WASM/HTML pipeline works.
pub struct StubEngine {
    pub processed_source: String,
    pub error: Option<String>,
}

impl StubEngine {
    pub fn new() -> Self {
        Self {
            processed_source: String::new(),
            error: None,
        }
    }
}

impl LuaEngine for StubEngine {
    fn load_cart(&mut self, _state: &mut PicoState, cart: &Cart) -> Result<(), String> {
        self.processed_source = preprocessor::preprocess(&cart.lua_code);
        self.error = None;
        Ok(())
    }

    fn call_init(&mut self, state: &mut PicoState) {
        // Show a placeholder screen so you can see the engine booted.
        crate::gfx::cls(state, 1);
        let lines: &[&[u8]] = &[
            b"PICO-R",
            b"",
            b"PURE-WASM RUST PORT",
            b"OF PICO-Z.",
            b"",
            b"CART LOADED:",
        ];
        for (i, line) in lines.iter().enumerate() {
            crate::gfx::draw_text(&mut state.memory, line, 8, 8 + i as i32 * 8, 7);
        }
        let preview_len = core::cmp::min(self.processed_source.len(), 18);
        let preview = &self.processed_source.as_bytes()[..preview_len];
        crate::gfx::draw_text(&mut state.memory, preview, 8, 8 + 6 * 8, 12);
        crate::gfx::draw_text(
            &mut state.memory,
            b"NO LUA VM WIRED.",
            8,
            8 + 8 * 8,
            10,
        );
    }

    fn call_update(&mut self, _state: &mut PicoState) {}
    fn call_draw(&mut self, _state: &mut PicoState) {}

    fn use_60fps(&self) -> bool {
        false
    }
    fn had_error(&self) -> bool {
        self.error.is_some()
    }
    fn error_message(&self) -> &str {
        self.error.as_deref().unwrap_or("")
    }
    fn save_globals(&self) -> Vec<u8> { Vec::new() }
    fn load_globals(&mut self, _data: &[u8]) -> Result<(), String> { Ok(()) }
}
