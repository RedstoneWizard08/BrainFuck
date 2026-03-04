use bf::parse;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

pub const HELLO_WORLD: &str = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";

fn hello_world(c: &mut Criterion) {
    c.bench_function("parse: hello world", |b| {
        b.iter(|| parse(black_box(HELLO_WORLD)));
    });
}

criterion_group!(benches, hello_world);
criterion_main!(benches);
