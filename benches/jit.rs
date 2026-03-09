use criterion::{Criterion, criterion_main};
use std::hint::black_box;

macro_rules! opts {
    ($level: expr) => {
        bf::compiler::CompilerOptions {
            opt_level: $level,
            ..Default::default()
        }
    };
}

macro_rules! jit_single_bench {
    (compile; $c: ident: $name: ident [$level: expr] = $display: expr) => {
        paste::paste! {
            fn [<compile_ $name _opt_ $level>](c: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>) {
                let opts = opts!($level);
                let program = bf::parse([<$name:upper>]);
                let program = bf::opt::Optimizer::new(&opts, program).run_all().finish();
                let io = bf::testing::BufTestingIo::new();

                c.bench_function(concat!("jit compile [opt_level=", stringify!($level), "]: ", $display), |b| {
                    b.iter(|| {
                        bf::compiler::cranelift::jit_compile(
                            black_box(&program),
                            black_box(opts.clone()),
                            black_box(Some(Box::new(&io))),
                        )
                    });
                });
            }

            [<compile_ $name _opt_ $level>](&mut $c);
        }
    };

    (run; $c: ident: $name: ident [$level: expr] = $display: expr) => {
        paste::paste! {
            fn [<run_ $name _opt_ $level>](c: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>) {
                let opts = opts!($level);
                let program = bf::parse([<$name:upper>]);
                let program = bf::opt::Optimizer::new(&opts, program).run_all().finish();
                let io = bf::testing::BufTestingIo::new();
                let func = bf::compiler::cranelift::jit_compile(&program, opts, Some(Box::new(&io)));

                c.bench_function(concat!("jit run [opt_level=", stringify!($level), "]: ", $display), |b| {
                    b.iter(|| func());
                });
            }

            [<run_ $name _opt_ $level>](&mut $c);
        }
    };
}

macro_rules! jit_bench {
    ($c: ident: $name: ident = $display: expr) => {
        jit_single_bench!(compile; $c: $name [0] = $display);
        jit_single_bench!(run; $c: $name [0] = $display);

        jit_single_bench!(compile; $c: $name [8] = $display);
        jit_single_bench!(run; $c: $name [8] = $display);
    };
}

pub const HELLO_WORLD: &str = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
pub const MANDELBROT: &str = include_str!("../tests/fixtures/basic/Mandelbrot.b");

/// The function which runs the benchmarks.
pub fn benches() {
    let mut c = Criterion::default().configure_from_args();

    let mut hello = c.benchmark_group("jit-hello-world");

    jit_bench!(hello: hello_world = "hello world");
    hello.finish();

    let mut mand = c.benchmark_group("jit-mandelbrot");

    mand.sample_size(20);
    jit_bench!(mand: mandelbrot = "mandelbrot");
    mand.finish();
}

criterion_main!(benches);
