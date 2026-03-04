## Hardware

- **CPU:** Intel Core i7-1165G7 @ 4.7 GHz
- **RAM:** 16 GB LPDDR4 @ 4267 MT/s
- **Kernel:** Linux 6.18.2-2-cachyos

## Benchmarks

> `tests/fixtures/Mandelbrot-extreme.b`

- **Compiled with:** `cargo run --release a tests/fixtures/Mandelbrot-extreme.b --unsafe-mode -O8`
- **Result:** 1 minute, 55 seconds

> `tests/fixtures/Mandelbrot.b`

- **Compiled with:** `cargo run --release aot --unsafe-mode -O8`

```sh
Benchmark 1: ./a.out
  Time (mean ± σ):     919.7 ms ±  12.9 ms    [User: 913.9 ms, System: 1.0 ms]
  Range (min … max):   904.9 ms … 937.5 ms    10 runs
```

> `tests/fixtures/Beer.b`

- **Compiled with:** `cargo run --release aot --unsafe-mode -O8`

```sh
Benchmark 1: ./a.out
  Time (mean ± σ):     457.4 µs ± 334.0 µs    [User: 403.9 µs, System: 440.4 µs]
  Range (min … max):   174.2 µs … 5442.7 µs    1820 runs
```

> `tests/fixtures/Bench.b`

- **Compiled with:** `cargo run --release aot --unsafe-mode -O8`

```sh
Benchmark 1: ./a.out
  Time (mean ± σ):     962.8 µs ± 551.8 µs    [User: 765.4 µs, System: 462.0 µs]
  Range (min … max):   471.3 µs … 8150.8 µs    1804 runs
```

> `tests/fixtures/chess.b`

- **Compiled with:** `cargo run --release aot --unsafe-mode -O8`

```sh
Benchmark 1: ./a.out
  Time (mean ± σ):     486.2 µs ± 278.1 µs    [User: 370.7 µs, System: 485.0 µs]
  Range (min … max):   174.3 µs … 5388.3 µs    1797 runs
```

> `tests/fixtures/Skiploop.b`

- **Compiled with:** `cargo run --release aot --unsafe-mode -O8`

```sh
Benchmark 1: ./a.out
  Time (mean ± σ):     407.3 µs ± 270.4 µs    [User: 330.4 µs, System: 486.5 µs]
  Range (min … max):   128.8 µs … 4085.4 µs    2171 runs
```
