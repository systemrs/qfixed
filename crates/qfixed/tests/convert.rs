//! Tests for `Q`/`UQ` conversions: widen/truncate/saturate, signed/unsigned,
//! `From<integer>`, and reformat.

use qfixed::typenum::consts::*;
use qfixed::{Q, UQ};

// ---------------------------------------------------------------------------
// Q: Widen / truncate / saturate
// ---------------------------------------------------------------------------

/// Verifies lossless widening from Q12.4 to Q16.16.
#[test]
fn q_widen() {
    let a = Q::<U12, U4>::from_f64(16.5);
    let b: Q<U16, U16> = a.widen();
    assert!((b.to_f64() - 16.5).abs() < 1e-4);
}

/// Verifies truncation from Q16.16 to Q12.4 preserves value.
#[test]
fn q_truncate() {
    let a = Q::<U16, U16>::from_f64(16.5);
    let b: Q<U12, U4> = a.truncate();
    assert!((b.to_f64() - 16.5).abs() < 0.1);
}

/// Verifies saturation clamps positive overflow to MAX.
#[test]
fn q_saturate() {
    let a = Q::<U16, U16>::from_int(1000);
    let b: Q<U4, U4> = a.saturate();
    assert_eq!(b, Q::<U4, U4>::MAX);
}

/// Verifies saturation clamps negative overflow to MIN.
#[test]
fn q_saturate_negative() {
    let a = Q::<U16, U16>::from_int(-1000);
    let b: Q<U4, U4> = a.saturate();
    assert_eq!(b, Q::<U4, U4>::MIN);
}

// ---------------------------------------------------------------------------
// Signed <-> unsigned conversions
// ---------------------------------------------------------------------------

/// Verifies UQ0.16 to Q16.16 signed conversion preserves value.
#[test]
fn uq_to_signed() {
    let v = UQ::<U0, U16>::from_f64(0.5);
    let s: Q<U16, U16> = v.to_signed();
    assert!((s.to_f64() - 0.5).abs() < 0.001);
}

/// Verifies Q4.12 to UQ4.12 unsigned conversion preserves positive
/// values.
#[test]
fn q_to_unsigned() {
    let v = Q::<U4, U12>::from_f64(0.75);
    let u: UQ<U4, U12> = v.to_unsigned();
    assert!((u.to_f64() - 0.75).abs() < 0.001);
}

/// Verifies negative Q values clamp to UQ::ZERO on unsigned conversion.
#[test]
fn q_to_unsigned_clamps_negative() {
    let v = Q::<U4, U12>::from_int(-1);
    let u: UQ<U4, U12> = v.to_unsigned();
    assert_eq!(u, UQ::<U4, U12>::ZERO);
}

// ---------------------------------------------------------------------------
// From<integer> conversions
// ---------------------------------------------------------------------------

/// Verifies `From<i32>` for Q16.16.
#[test]
fn q_from_i32() {
    let v: Q<U16, U16> = Q::from(42i32);
    assert_eq!(v.to_int(), 42);
}

/// Verifies `From<u8>` for UQ8.8.
#[test]
fn uq_from_u8() {
    let v: UQ<U8, U8> = UQ::from(5u8);
    assert_eq!(v.to_int(), 5);
}

// ---------------------------------------------------------------------------
// Reformat
// ---------------------------------------------------------------------------

/// Verifies reformat from Q12.4 to Q4.12 preserves value.
#[test]
fn q_reformat() {
    let a = Q::<U12, U4>::from_f64(2.5);
    let b: Q<U4, U12> = a.reformat();
    assert!((b.to_f64() - 2.5).abs() < 0.01);
}
