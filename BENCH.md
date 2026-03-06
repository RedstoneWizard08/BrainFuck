## Hardware

- **CPU:** Intel Core i7-1165G7 @ 4.7 GHz
- **RAM:** 16 GB LPDDR4 @ 4267 MT/s
- **Kernel:** Linux 6.18.2-2-cachyos

## Test Benchmarks

> `tests/fixtures/Mandelbrot-extreme.b`

- **Compiled with:** `cargo run --release a tests/fixtures/Mandelbrot-extreme.b --unsafe-mode -O8`
- **Result:** 1 minute, 55 seconds

> `tests/fixtures/Mandelbrot.b`

- **Compiled with:** `cargo run --release aot --unsafe-mode -O8`

```sh
Benchmark 1: ./a.out
  Time (mean ± σ):     862.6 ms ± 131.2 ms    [User: 857.8 ms, System: 0.6 ms]
  Range (min … max):   753.9 ms … 1022.4 ms    10 runs
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

> `tests/fixtures/squaresums.b`

- **Compiled with:** `cargo run --release aot --unsafe-mode -O8`

```sh
Benchmark 1: ./a.out
  Time (mean ± σ):     421.6 µs ± 232.5 µs    [User: 345.8 µs, System: 457.9 µs]
  Range (min … max):   154.0 µs … 3170.5 µs    1760 runs
```

> `tests/fixtures/Tribit.b`

- **Compiled with:** `cargo run --release aot --unsafe-mode -O8`

```sh
Benchmark 1: ./a.out
  Time (mean ± σ):     363.0 µs ± 505.6 µs    [User: 381.8 µs, System: 392.7 µs]
  Range (min … max):     3.1 µs … 9742.2 µs    1889 runs
```

> `tests/fixtures/Mandelbrot-tiny.b`

- **Compiled with:** `cargo run --release aot --unsafe-mode -O8`

```sh
Benchmark 1: ./a.out
  Time (mean ± σ):     248.6 ms ±  18.1 ms    [User: 245.9 ms, System: 0.9 ms]
  Range (min … max):   232.0 ms … 284.6 ms    12 runs
``

> `tests/fixtures/Mandelbrot-r64.b`

- **Compiled with:** `cargo run --release aot --unsafe-mode -O8``

```sh
Benchmark 1: ./a.out
  Time (mean ± σ):     940.0 ms ±  21.5 ms    [User: 932.8 ms, System: 1.3 ms]
  Range (min … max):   911.0 ms … 980.1 ms    10 runs
```

> `tests/fixtures/Hanoi.b`

- **Compiled with:** `cargo run --release aot --unsafe-mode -O8``

```sh
Benchmark 1: ./a.out
  Time (mean ± σ):      18.4 ms ±   2.2 ms    [User: 17.3 ms, System: 0.8 ms]
  Range (min … max):    16.5 ms …  32.3 ms    118 runs
```

> `tests/fixtures/Golden.b`

- **Compiled with:** `cargo run --release aot --unsafe-mode -O8``

```sh
Benchmark 1: ./a.out
  Time (mean ± σ):      17.9 ms ±   2.7 ms    [User: 16.7 ms, System: 0.8 ms]
  Range (min … max):    15.7 ms …  34.7 ms    178 runs
```
