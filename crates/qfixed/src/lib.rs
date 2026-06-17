//! Bit-accurate fixed-point types with Q-notation for RTL digital twin modeling.
//!
//! Provides `Q<I, F>` (signed) and `UQ<I, F>` (unsigned) fixed-point types
//! where `I` is the number of integer bits (including sign for `Q`) and `F`
//! is the number of fractional bits. `I` and `F` are [`typenum`] type-level
//! unsigned integers (`U0`, `U1`, `U8`, …), e.g. `Q<U12, U4>`.
//! Total bit width is `I + F`, backed by `i64`/`u64` internally.
//!
//! `CQ<I, F>` is a signed complex type — a real/imaginary `Q<I, F>` pair —
//! for bit-accurate modeling of IQ-sample and complex datapaths. Complex
//! multiply grows by one integer bit more than a real multiply, since each
//! component is a sum or difference of two products:
//! - `CQ<I,F> + CQ<I,F>` → `CQ<Sum<I, U1>, F>`
//! - `CQ<I,F> * CQ<I,F>` → `CQ<Sum<Sum<I, I>, U1>, Sum<F, F>>`
//! - `CQ<I,F> * Q<I,F>` (scalar) → `CQ<Sum<I, I>, Sum<F, F>>`
//!
//! Arithmetic operators produce widened outputs that cannot overflow. The
//! output widths are computed at the type level with `typenum` (`Sum<I, U1>` is
//! `I + 1`, `Sum<I, I>` is `2 * I`), so this works on stable Rust:
//! - `Q<I,F> + Q<I,F>` → `Q<Sum<I, U1>, F>`
//! - `Q<I,F> - Q<I,F>` → `Q<Sum<I, U1>, F>`
//! - `Q<I,F> * Q<I,F>` → `Q<Sum<I, I>, Sum<F, F>>`
//! - `-Q<I,F>` → `Q<Sum<I, U1>, F>`
//! - `UQ<I,F> - UQ<I,F>` → `Q<Sum<I, U1>, F>` (signed, since result can be negative)
//!
//! Use `.truncate()` or `.saturate()` to narrow results back down.
//! Use `wrapping_add`/`wrapping_sub`/`wrapping_mul` for same-type RTL
//! truncation semantics.
//! Use `saturating_add`/`saturating_sub`/`saturating_mul` to clamp.
//!
//! Reduce many values with [`core::iter::Sum`] into a caller-chosen wider
//! accumulator: summing `N` values grows the integer part by only
//! `ceil(log2(N))` bits (64 values → 6 bits), not one bit per addition. The
//! same applies to `UQ` and `CQ`. Use the `try_sum` associated function for a
//! checked reduction that returns [`FixedError`] instead of wrapping when the
//! chosen accumulator is too narrow.
//!
//! ```
//! use qfixed::Q;
//! use qfixed::typenum::{U4, U12};
//!
//! let a = Q::<U12, U4>::from(3i32);
//! let b = Q::<U12, U4>::from(4i32);
//! let sum = a + b; // Q<U13, U4> — one extra integer bit, cannot overflow
//! assert_eq!(sum.to_f64(), 7.0);
//! ```

#![no_std]
#![deny(unsafe_code)]
#![cfg_attr(all(not(debug_assertions), not(test)), deny(clippy::all))]
#![cfg_attr(all(not(debug_assertions), not(test)), deny(clippy::pedantic))]
#![cfg_attr(all(not(debug_assertions), not(test)), deny(missing_docs))]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
// Bit-accurate fixed-point arithmetic is built on deliberate narrowing,
// wrapping, and sign-changing casts that model RTL register semantics, so these
// pedantic cast lints would fire on essentially every operation.
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_lossless)]
// These are value types; constructors and operators return new values by
// design. Tagging every one `#[must_use]` is noise (cf. `must_use_candidate`).
#![allow(clippy::return_self_not_must_use)]
// `#[inline(always)]` on the trivial compile-time parameter checks is intended.
#![allow(clippy::inline_always)]
// Until 1.0.0, allow dead code and unused dependency warnings.
#![allow(dead_code)]
#![allow(unused_crate_dependencies)]

mod convert;
mod cq;
mod error;
mod fmt;
mod ops;
mod q;
#[cfg(feature = "serde")]
mod serde_impl;
mod uq;

pub use cq::CQ;
pub use error::FixedError;
pub use q::Q;
pub use uq::UQ;

/// Re-export of [`typenum`], whose type-level unsigned integers (`U0`, `U1`,
/// `U8`, …) supply the `I` and `F` parameters, e.g. `Q<U12, U4>`.
///
/// The widening operators derive their output widths from these at the type
/// level on stable Rust — `Q<I,F> + Q<I,F>` yields `Q<Sum<I, U1>, F>`.
pub use typenum;
