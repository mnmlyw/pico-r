use pico_r::{cart, preprocessor, state::PicoState};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        eprintln!("usage: dump-pp <cart.p8|cart.p8.png> [output.lua]");
        std::process::exit(1);
    }

    let input = &args[1];
    let output = args
        .get(2)
        .map(String::as_str)
        .unwrap_or("/tmp/pico-r_preprocessed.lua");

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
    let p = preprocessor::preprocess(&cart.lua_code);
    if let Err(e) = std::fs::write(output, &p) {
        eprintln!("write {}: {}", output, e);
        std::process::exit(4);
    }
    eprintln!("wrote {} bytes to {}", p.len(), output);
}
