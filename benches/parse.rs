//! Benchmarks for Brainf*ck program parsing.

use bf::parse;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

pub const HELLO_WORLD: &str = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
pub const MANDELBROT: &str = include_str!("../tests/fixtures/basic/Mandelbrot.b");

fn parser(c: &mut Criterion) {
    c.bench_function("parse: hello world", |b| {
        b.iter(|| parse(black_box(HELLO_WORLD)));
    });

    c.bench_function("parse: mandelbrot", |b| {
        b.iter(|| parse(black_box(MANDELBROT)));
    });
}

criterion_group!(benches, parser);
criterion_main!(benches);
