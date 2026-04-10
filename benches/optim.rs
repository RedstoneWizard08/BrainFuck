use bf::opt::v1::Optimizer;
use criterion::criterion_main;

pub const HELLO_WORLD: &str = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
pub const MANDELBROT: &str = include_str!("../tests/fixtures/basic/Mandelbrot.b");

macro_rules! optim_bench {
    ($c: ident; $name: ident [$level: expr] = $display: expr) => {
        $c.bench_function(
            concat!("optimize [level=", stringify!($level), "]: ", $display),
            |b| {
                let opts = bf::backend::CompilerOptions {
                    opt_level: $level,
                    ..Default::default()
                };

                let prog = bf::parse(pastey::paste! { [<$name:upper>] });

                b.iter(|| {
                    std::hint::black_box(Optimizer::new(
                        std::hint::black_box(&opts),
                        std::hint::black_box(prog.clone()),
                    ))
                    .run_all()
                });
            },
        );
    };
}

macro_rules! optim_benches {
    ($c: ident; $name: ident = $display: expr) => {{
        let mut g = $c.benchmark_group(concat!("optim-", stringify!($name)));

        optim_bench!(g; hello_world [0] = $display);
        optim_bench!(g; hello_world [8] = $display);

        g.finish();
    }}
}

/// The function which runs the benchmarks.
pub fn benches() {
    let mut c = criterion::Criterion::default().configure_from_args();

    optim_benches!(c; hello_world = "hello world");
    optim_benches!(c; mandelbrot = "mandelbrot");
}

criterion_main!(benches);
