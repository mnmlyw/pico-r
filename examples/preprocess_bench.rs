// Standalone microbench: `cargo run --release --example preprocess_bench`.
//
// Stays out of `cargo test` / `cargo bench` so we don't pull a panic=unwind
// dev-dep (criterion) that would clash with the panic=abort release deps.

use std::time::Instant;

fn run(name: &str, src: &str, iters: u32) {
    let mut total_ns: u128 = 0;
    let mut total_bytes: u128 = 0;
    for _ in 0..iters {
        let t = Instant::now();
        let out = pico_r::preprocessor::preprocess(src.as_bytes());
        total_ns += t.elapsed().as_nanos();
        total_bytes += out.len() as u128;
    }
    let ns_per_iter = total_ns / iters as u128;
    println!(
        "{name:32} {iters:>5}x {:>10}ns/iter  out={total_bytes}b",
        ns_per_iter
    );
}

fn main() {
    let short_if = "if (a==1) and (b==2) and (c==3) then x+=1 end\n".repeat(500);
    let compound = "x+=1\ny-=2\nz*=3\nw/=4\nv\\=5\nu%=6\nt^^=7\n".repeat(300);
    let hello = include_str!("../tests/fixtures/hello.p8");

    run("short_if_500", &short_if, 200);
    run("compound_assign_300", &compound, 200);
    run("hello_p8", hello, 1000);
}
