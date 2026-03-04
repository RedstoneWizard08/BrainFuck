use std::io::Cursor;

use bf::{interp::interpret, optimizer::Optimizer, parse};
use criterion::{Criterion, criterion_group, criterion_main};

pub const HELLO_WORLD: &str = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";

fn hello_world(c: &mut Criterion) {
    let program = parse(HELLO_WORLD);
    let program = Optimizer::new(program).run_all(1, false).finish();

    c.bench_function("interpret: hello world", |b| {
        b.iter(|| {
            let mut dummy = Vec::new();
            let mut dummy2 = Cursor::new(Vec::new());

            interpret(&program, &mut dummy, &mut dummy2);
        });
    });
}

criterion_group!(benches, hello_world);
criterion_main!(benches);
