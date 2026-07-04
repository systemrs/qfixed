# qfixed

[![crates.io](https://img.shields.io/crates/v/qfixed.svg?logo=rust)](https://crates.io/crates/qfixed)
[![docs.rs](https://img.shields.io/docsrs/qfixed?logo=docsdotrs)](https://docs.rs/qfixed)
[![CI](https://github.com/systemrs/qfixed/actions/workflows/ci.yml/badge.svg)](https://github.com/systemrs/qfixed/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/crates/msrv/qfixed.svg?logo=rust)](https://www.rust-lang.org)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/qfixed.svg)](#license)

Bit-accurate fixed-point arithmetic types using Texas Instruments style
**Q notation**, for RTL digital-twin modeling. `#![no_std]`; the only required
dependency is [`typenum`](https://docs.rs/typenum).

```sh
cargo add qfixed
```

## Types

- `Q<I, F>` — signed fixed-point with `I` integer bits (including sign) and `F` fractional bits.
- `UQ<I, F>` — unsigned fixed-point.
- `CQ<I, F>` — signed complex fixed-point (a real/imaginary `Q<I, F>` pair) for bit-accurate IQ-sample and complex-datapath modeling.

`I` and `F` are [`typenum`](https://docs.rs/typenum) type-level integers (`U0`, `U1`,
`U8`, …). Arithmetic operators return widened output types that cannot overflow; use
`.truncate()` or `.saturate()` to narrow back down, or the `wrapping_*` /
`saturating_*` methods for same-width RTL semantics.

```rust
use qfixed::Q;
use qfixed::typenum::{U4, U12};

let a = Q::<U12, U4>::from(3i32);
let b = Q::<U12, U4>::from(4i32);
let sum = a + b; // Q<U13, U4> — one extra integer bit, cannot overflow
assert_eq!(sum.to_f64(), 7.0);
```

## Compact storage

The packed containers store bulk arrays in the smallest integer that fits each
value's bit width, without changing the scalar types:

- `PackedArray<T, N>` / `PackedVec<T>` — byte-granular, up to 8× smaller than `[T; N]` for ≤ 8-bit types.
- `PackedBits<T, BYTES>` / `PackedBitsVec<T>` — bit-exact (e.g. 12-bit ADC samples pack at 12 bits, not 16).

Packing is lossless; the `*Vec` containers need the `alloc` feature.

## Feature flags

- `alloc` (off by default) — the growable packed containers (`PackedVec`,
  `PackedBitsVec`). Pulls in the `alloc` crate only; no external dependency.
- `serde` (off by default) — `Serialize`/`Deserialize` for the fixed-point types.

## License

Licensed under either of
[Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0) or
[MIT license](https://opensource.org/licenses/MIT) at your option.
