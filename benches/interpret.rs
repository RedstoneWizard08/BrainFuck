//! Benchmarks for the interpreter performance.

use bf::{backend::CompilerOptions, interp::interpret, parse};
use criterion::{Criterion, criterion_group, criterion_main};
use std::io::Cursor;

pub const HELLO_WORLD: &str = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";

fn hello_world(c: &mut Criterion) {
    let program = parse(HELLO_WORLD);

    let program = bf::opt::v2::optimize_v2(
        &program,
        &CompilerOptions {
            opt_level: 8,
            ..Default::default()
        },
    );

    let mut dummy = Vec::new();
    let mut dummy2 = Cursor::new(Vec::new());

    c.bench_function("interpret: hello world", |b| {
        b.iter(|| {
            interpret(&program, &mut dummy, &mut dummy2);
        });
    });
}

criterion_group!(benches, hello_world);
criterion_main!(benches);
