use std::io::Cursor;

use bf::{interp::interpret, opt::Optimizer, parse};
use criterion::{Criterion, criterion_group, criterion_main};

pub const HELLO_WORLD: &str = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";

fn hello_world(c: &mut Criterion) {
    let program = parse(HELLO_WORLD);

    let program = Optimizer::new(&Default::default(), program)
        .run_all()
        .finish();

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
