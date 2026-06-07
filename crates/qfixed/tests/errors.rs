//! Tests for the error-handling model: wrapping/defined infallible defaults,
//! `try_*` checked conversions, and `checked_div`.

use qfixed::typenum::consts::*;
use qfixed::{CQ, FixedError, Q, UQ};

// ---------------------------------------------------------------------------
// from_count: wrapping default + try_from_count
// ---------------------------------------------------------------------------

/// `from_count` wraps an out-of-range count instead of panicking.
#[test]
fn q_from_count_wraps() {
    // Q4.4 is 8-bit signed: range [-128, 127]. Count 128 wraps to -128.
    assert_eq!(Q::<U4, U4>::from_count(128).to_count(), -128);
}

/// `try_from_count` returns Ok in range and Err out of range.
#[test]
fn q_try_from_count() {
    assert_eq!(
        Q::<U4, U4>::try_from_count(127).map(|q| q.to_count()),
        Ok(127)
    );
    assert_eq!(
        Q::<U4, U4>::try_from_count(128),
        Err(FixedError::OutOfRange)
    );
    assert_eq!(
        Q::<U4, U4>::try_from_count(-129),
        Err(FixedError::OutOfRange)
    );
}

/// UQ `from_count` wraps; `try_from_count` checks.
#[test]
fn uq_from_count_and_try() {
    assert_eq!(UQ::<U4, U4>::from_count(256).to_count(), 0); // wraps mod 256
    assert_eq!(
        UQ::<U4, U4>::try_from_count(255).map(|q| q.to_count()),
        Ok(255)
    );
    assert_eq!(
        UQ::<U4, U4>::try_from_count(256),
        Err(FixedError::OutOfRange)
    );
}

// ---------------------------------------------------------------------------
// from_bits: masking default + try_from_bits
// ---------------------------------------------------------------------------

/// `from_bits` masks bits above the low `I + F`; `try_from_bits` errors on them.
#[test]
fn q_from_bits_masks_and_try() {
    // Q4.4 mask is 0xFF; 0x1F0 masks to 0xF0 (count -16).
    assert_eq!(Q::<U4, U4>::from_bits(0x1F0).to_count(), -16);
    assert!(Q::<U4, U4>::try_from_bits(0xF0).is_ok());
    assert_eq!(
        Q::<U4, U4>::try_from_bits(0x1F0),
        Err(FixedError::OutOfRange)
    );
}

// ---------------------------------------------------------------------------
// try_from_f64
// ---------------------------------------------------------------------------

/// `try_from_f64` rejects NaN/infinite and out-of-range, accepts valid.
#[test]
fn q_try_from_f64() {
    assert!(Q::<U8, U8>::try_from_f64(1.5).is_ok());
    assert_eq!(
        Q::<U8, U8>::try_from_f64(f64::NAN),
        Err(FixedError::NotFinite)
    );
    assert_eq!(
        Q::<U8, U8>::try_from_f64(f64::INFINITY),
        Err(FixedError::NotFinite)
    );
    assert_eq!(Q::<U8, U8>::try_from_f64(1e9), Err(FixedError::OutOfRange));
}

/// UQ `try_from_f64` rejects negative values.
#[test]
fn uq_try_from_f64_negative() {
    assert_eq!(
        UQ::<U8, U8>::try_from_f64(-0.5),
        Err(FixedError::OutOfRange)
    );
    assert!(UQ::<U8, U8>::try_from_f64(0.5).is_ok());
}

// ---------------------------------------------------------------------------
// checked_div
// ---------------------------------------------------------------------------

/// `checked_div` returns None on divide-by-zero, Some otherwise.
#[test]
fn q_checked_div() {
    let a = Q::<U8, U8>::from(7i32);
    let b = Q::<U8, U8>::from(2i32);
    assert_eq!(a.checked_div(b).map(|q| q.to_f64()), Some(3.5));
    assert_eq!(a.checked_div(Q::<U8, U8>::ZERO), None);
}

/// UQ `checked_div` returns None on divide-by-zero.
#[test]
fn uq_checked_div() {
    let a = UQ::<U8, U8>::from(7u32);
    assert_eq!(a.checked_div(UQ::<U8, U8>::ZERO), None);
    assert_eq!(
        a.checked_div(UQ::<U8, U8>::from(2u32)).map(|q| q.to_f64()),
        Some(3.5)
    );
}

// ---------------------------------------------------------------------------
// to_signed: wrapping default + try_to_signed
// ---------------------------------------------------------------------------

/// `to_signed` wraps out-of-range; `try_to_signed` errors, and a fitting value
/// converts cleanly.
#[test]
fn uq_to_signed_and_try() {
    // UQ8.8 value 200.0 does not fit Q8.8 (max ~127.996).
    let big = UQ::<U8, U8>::from(200u32);
    assert_eq!(big.try_to_signed::<U8, U8>(), Err(FixedError::OutOfRange));

    let ok = UQ::<U8, U8>::from(100u32);
    assert_eq!(ok.try_to_signed::<U8, U8>().map(|q| q.to_f64()), Ok(100.0));
}

// ---------------------------------------------------------------------------
// CQ try_* variants
// ---------------------------------------------------------------------------

/// CQ `try_from_count` errors if either component is out of range.
#[test]
fn cq_try_from_count() {
    assert!(CQ::<U4, U4>::try_from_count(10, -10).is_ok());
    assert_eq!(
        CQ::<U4, U4>::try_from_count(200, 0),
        Err(FixedError::OutOfRange)
    );
}

/// CQ `try_from_f64` rejects non-finite components.
#[test]
fn cq_try_from_f64() {
    assert!(CQ::<U8, U8>::try_from_f64(1.0, -1.0).is_ok());
    assert_eq!(
        CQ::<U8, U8>::try_from_f64(1.0, f64::NAN),
        Err(FixedError::NotFinite)
    );
}

// ---------------------------------------------------------------------------
// try_narrow: value-preserving (exact or error)
// ---------------------------------------------------------------------------

/// `try_narrow` succeeds only when the value is exactly representable;
/// distinguishes precision loss (Inexact) from magnitude overflow (OutOfRange).
#[test]
fn q_try_narrow_exact() {
    // Q16.16 -> Q4.4 (range [-8, 8)). 1.5 = 24/16 is exact in Q4.4.
    let exact = Q::<U16, U16>::from_f64(1.5);
    assert_eq!(exact.try_narrow::<U4, U4>().map(|q| q.to_f64()), Ok(1.5));

    // Fractional bits below Q4.4's resolution would be lost -> Inexact.
    let frac = Q::<U16, U16>::from_f64(1.0 + 1.0 / 4096.0);
    assert_eq!(frac.try_narrow::<U4, U4>(), Err(FixedError::Inexact));

    // A magnitude that overflows the target -> OutOfRange.
    let big = Q::<U16, U16>::from(1000i32);
    assert_eq!(big.try_narrow::<U4, U4>(), Err(FixedError::OutOfRange));
}

/// UQ `try_narrow` is likewise exact-or-error.
#[test]
fn uq_try_narrow_exact() {
    let exact = UQ::<U16, U16>::from_f64(1.5);
    assert_eq!(exact.try_narrow::<U4, U4>().map(|q| q.to_f64()), Ok(1.5));

    let frac = UQ::<U16, U16>::from_f64(1.0 + 1.0 / 4096.0);
    assert_eq!(frac.try_narrow::<U4, U4>(), Err(FixedError::Inexact));

    let big = UQ::<U16, U16>::from(1000u32);
    assert_eq!(big.try_narrow::<U4, U4>(), Err(FixedError::OutOfRange));
}

/// CQ `try_narrow` errors if either component is not exactly representable.
#[test]
fn cq_try_narrow_exact() {
    let ok = CQ::<U16, U16>::from_f64(1.5, -1.5);
    assert!(ok.try_narrow::<U4, U4>().is_ok());

    // Imaginary part overflows -> OutOfRange.
    let big = CQ::<U16, U16>::from_f64(1.5, 100.0);
    assert_eq!(big.try_narrow::<U4, U4>(), Err(FixedError::OutOfRange));

    // Real part loses precision -> Inexact.
    let frac = CQ::<U16, U16>::from_f64(1.0 + 1.0 / 4096.0, 0.0);
    assert_eq!(frac.try_narrow::<U4, U4>(), Err(FixedError::Inexact));
}

// ---------------------------------------------------------------------------
// round_to_zero: deliberate quantization toward zero (reduces magnitude)
// ---------------------------------------------------------------------------

/// `round_to_zero` reduces magnitude (toward zero); `truncate` floors toward
/// −∞. They agree for non-negative values and differ for negatives.
#[test]
fn q_round_to_zero() {
    // Q16.16 -> Q16.0
    let pos = Q::<U16, U16>::from_f64(2.7);
    assert_eq!(pos.round_to_zero::<U16, U0>().to_f64(), 2.0);
    assert_eq!(pos.truncate::<U16, U0>().to_f64(), 2.0); // same for positives

    let neg = Q::<U16, U16>::from_f64(-2.7);
    assert_eq!(neg.round_to_zero::<U16, U0>().to_f64(), -2.0); // toward zero
    assert_eq!(neg.truncate::<U16, U0>().to_f64(), -3.0); // floor toward -inf
}

/// UQ `round_to_zero` coincides with `truncate` (no negatives).
#[test]
fn uq_round_to_zero() {
    let v = UQ::<U16, U16>::from_f64(2.7);
    assert_eq!(v.round_to_zero::<U16, U0>().to_f64(), 2.0);
    assert_eq!(v.truncate::<U16, U0>().to_f64(), 2.0);
}

/// CQ `round_to_zero` is componentwise.
#[test]
fn cq_round_to_zero() {
    let v = CQ::<U16, U16>::from_f64(2.7, -2.7);
    let r: CQ<U16, U0> = v.round_to_zero();
    assert_eq!(r.re.to_f64(), 2.0);
    assert_eq!(r.im.to_f64(), -2.0);
}
