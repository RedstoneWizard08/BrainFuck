use bf::{compiler::CompilerOptions, opt::Optimizer, parse};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

pub const HELLO_WORLD: &str = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
pub const MANDELBROT: &str = include_str!("../tests/fixtures/basic/Mandelbrot.b");

fn hello_world(c: &mut Criterion) {
    let program = parse(HELLO_WORLD);
    let opts = CompilerOptions::default();

    c.bench_function("optimize: hello world", |b| {
        b.iter(|| {
            Optimizer::new(black_box(&opts), black_box(program.clone()))
                .run_all()
                .finish()
        });
    });
}

fn hello_world_8x(c: &mut Criterion) {
    let program = parse(HELLO_WORLD);

    let opts = CompilerOptions {
        opt_level: 8,
        ..Default::default()
    };

    c.bench_function("optimize [8x]: hello world", |b| {
        b.iter(|| {
            Optimizer::new(black_box(&opts), black_box(program.clone()))
                .run_all()
                .finish()
        });
    });
}

fn mandelbrot(c: &mut Criterion) {
    let program = parse(MANDELBROT);
    let opts = CompilerOptions::default();

    c.bench_function("optimize: mandelbrot", |b| {
        b.iter(|| {
            Optimizer::new(black_box(&opts), black_box(program.clone()))
                .run_all()
                .finish()
        });
    });
}

fn mandelbrot_8x(c: &mut Criterion) {
    let program = parse(MANDELBROT);

    let opts = CompilerOptions {
        opt_level: 8,
        ..Default::default()
    };

    c.bench_function("optimize [8x]: mandelbrot", |b| {
        b.iter(|| {
            Optimizer::new(black_box(&opts), black_box(program.clone()))
                .run_all()
                .finish()
        });
    });
}

criterion_group!(
    benches,
    hello_world,
    hello_world_8x,
    mandelbrot,
    mandelbrot_8x,
);

criterion_main!(benches);
