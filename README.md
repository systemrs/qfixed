# qfixed

[![CI](https://github.com/systemrs/qfixed/actions/workflows/ci.yml/badge.svg)](https://github.com/systemrs/qfixed/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/qfixed.svg?logo=rust)](https://crates.io/crates/qfixed)
[![docs.rs](https://img.shields.io/docsrs/qfixed?logo=docsdotrs)](https://docs.rs/qfixed)
[![MSRV](https://img.shields.io/crates/msrv/qfixed.svg?logo=rust)](https://www.rust-lang.org)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/qfixed.svg)](#license)

This repository implements fixed-point arithmetic types based on Texas Instruments style Q notation.

```sh
cargo add qfixed
```

## Types

- `Q<I, F>` — signed fixed-point with `I` integer bits (including sign) and `F` fractional bits.
- `UQ<I, F>` — unsigned fixed-point.
- `CQ<I, F>` — signed complex fixed-point (a real/imaginary `Q<I, F>` pair) for bit-accurate IQ-sample and complex-datapath modeling.

`I` and `F` are [`typenum`](https://docs.rs/typenum) type-level integers (`U0`, `U1`, `U8`, …). Arithmetic operators return widened output types that cannot overflow; use `.truncate()` or `.saturate()` to narrow back down, or the `wrapping_*` / `saturating_*` methods for same-width RTL semantics.

## Compact storage

The scalar types are always backed by an `i64`/`u64` (a `CQ` by two), which is convenient for arithmetic but wasteful for large buffers. The packed containers (re-exported at the crate root) store bulk arrays in the smallest integer that fits each value's bit width, without changing the scalar types or their `const fn` API:

- `PackedArray<T, N>` / `PackedVec<T>` — byte-granular: each element uses the smallest primitive (`i8`/`i16`/`i32`/`i64`), up to 8× smaller than `[T; N]` for ≤ 8-bit types. `PackedVec` needs the `alloc` feature.
- `PackedBits<T, BYTES>` / `PackedBitsVec<T>` — bit-exact: each element occupies exactly `I + F` bits (e.g. 12-bit ADC samples pack at 12 bits, not 16), at the cost of slower shift/mask access. `PackedBitsVec` needs the `alloc` feature.

Packing is lossless; elements are returned by value (`get`) and written with `set`.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
