use pico_r::{cart, state::PicoState};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        eprintln!("usage: dump-raw <cart.p8|cart.p8.png> [output.lua]");
        std::process::exit(1);
    }

    let input = &args[1];
    let output = args
        .get(2)
        .map(String::as_str)
        .unwrap_or("/tmp/raw_lua.lua");

    let data = match std::fs::read(input) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("read {}: {}", input, e);
            std::process::exit(2);
        }
    };
    let mut state = PicoState::new();
    let cart = match cart::load_bytes(&data, &mut state.memory) {
        Ok(cart) => cart,
        Err(e) => {
            eprintln!("load {}: {:?}", input, e);
            std::process::exit(3);
        }
    };
    if let Err(e) = std::fs::write(output, &cart.lua_code) {
        eprintln!("write {}: {}", output, e);
        std::process::exit(4);
    }
    eprintln!("wrote {} bytes to {}", cart.lua_code.len(), output);
}
