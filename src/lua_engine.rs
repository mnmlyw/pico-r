// LuaEngine is the small boundary between the emulator core and the hand-rolled
// PICO-8 Lua runtime in `pico_lua`. Keeping this as a trait keeps host tools
// and the WASM entry points decoupled from the concrete interpreter type.

use crate::cart::Cart;
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
