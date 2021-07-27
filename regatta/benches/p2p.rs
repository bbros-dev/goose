// Peer-to-Peer Benchmarks
// The p2p component performs these functions
// - Discover/forget Regatta peers
// - Connect/disconnect Regatta peers
// - Distribute VCR file to Regatta peers
//
// This benchmark records the performance of:
// 1. Launch Regatta triple (currently `p2p`)
// 2. Distribute VCR cassette
// 3. Read back distributed VCR cassette (correctness check)
// 4. Shutdown Regatta triple

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::io::Write;

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn setup_cassette() {
    let w = std::fs::File::create("./vcr.yml").unwrap();
    writeln!(&w, "A test\nActual content\nMore content\nAnother test").unwrap();
}

fn read_cassette() {
    let csstt = "Done".to_string();
    regatta::read_local_recipes();
}

fn criterion_benchmark(c: &mut Criterion) {
    setup_cassette();
    c.bench_function("vcr read", |b| b.iter(|| read_cassette()));
    // c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
