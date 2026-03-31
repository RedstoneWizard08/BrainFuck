use std::iter;

use bf::opt::base::{BfInsn, InsnBuf};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

pub fn insn_benches(c: &mut Criterion) {
    let mut g = c.benchmark_group("insn buf: noop");
    let sizes: [usize; _] = [100_000, 200_000, 300_000];

    for size in sizes {
        g.throughput(Throughput::Elements(size as u64));

        g.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let insns = iter::repeat(BfInsn::Noop).take(size).collect::<Vec<_>>();

            b.iter_with_setup(
                || InsnBuf::new(),
                |mut buf| {
                    buf.add_all_ref(&insns);
                },
            );
        });
    }
}

criterion_group!(benches, insn_benches);
criterion_main!(benches);
