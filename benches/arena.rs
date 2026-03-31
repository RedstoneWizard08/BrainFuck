use bf::opt::{arena::Arena, base::BfInsn};
use criterion::{Criterion, criterion_group, criterion_main};

pub fn arena_benches(c: &mut Criterion) {
    c.bench_function("arena alloc string", |b| {
        let arena = Arena::<String>::new();

        b.iter(|| arena.alloc());
    });

    c.bench_function("arena alloc insn", |b| {
        let arena = Arena::<BfInsn>::new();

        b.iter(|| arena.alloc());
    });
}

criterion_group!(benches, arena_benches);
criterion_main!(benches);
