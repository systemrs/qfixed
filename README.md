# qfixed
This repository implements fixed-point arithmetic types based on Texas Instruments style Q notation.

## Types

- `Q<I, F>` — signed fixed-point with `I` integer bits (including sign) and `F` fractional bits.
- `UQ<I, F>` — unsigned fixed-point.
- `CQ<I, F>` — signed complex fixed-point (a real/imaginary `Q<I, F>` pair) for bit-accurate IQ-sample and complex-datapath modeling.

`I` and `F` are [`typenum`](https://docs.rs/typenum) type-level integers (`U0`, `U1`, `U8`, …). Arithmetic operators return widened output types that cannot overflow; use `.truncate()` or `.saturate()` to narrow back down, or the `wrapping_*` / `saturating_*` methods for same-width RTL semantics.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
