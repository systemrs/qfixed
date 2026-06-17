//! Tests for `Q`/`UQ` arithmetic: operators, wrapping, saturating, widening,
//! shifts, and bitwise operations.

use qfixed::typenum::consts::*;
use qfixed::{Q, UQ};

// ---------------------------------------------------------------------------
// Q: Arithmetic (std::ops)
// ---------------------------------------------------------------------------

/// Verifies Q addition: 3 + 4 = 7.
#[test]
fn q_add() {
    let a = Q::<U12, U4>::from(3i32);
    let b = Q::<U12, U4>::from(4i32);
    let c = a + b;
    assert_eq!(c.to_f64(), 7.0);
}

/// Verifies Q subtraction: 10 - 3 = 7.
#[test]
fn q_sub() {
    let a = Q::<U12, U4>::from(10i32);
    let b = Q::<U12, U4>::from(3i32);
    let c = a - b;
    assert_eq!(c.to_f64(), 7.0);
}

/// Verifies Q negation: -5 = -(5).
#[test]
fn q_neg() {
    let a = Q::<U12, U4>::from(5i32);
    let b = -a;
    assert_eq!(b.to_f64(), -5.0);
}

/// Verifies that Q4.4 addition widens to Q5.4 (no overflow possible).
#[test]
fn q_add_widens() {
    let a = Q::<U4, U4>::MAX; // 7.9375
    let b = Q::<U4, U4>::from(1i32);
    let c: Q<U5, U4> = a + b;
    // Q5.4 has enough range — no overflow
    assert!((c.to_f64() - 8.9375).abs() < 1e-4);
}

// ---------------------------------------------------------------------------
// Q: Wrapping operations
// ---------------------------------------------------------------------------

/// Verifies wrapping_add wraps MAX + 1 LSB to MIN.
#[test]
fn q_wrapping_add() {
    let a = Q::<U4, U4>::MAX; // 7.9375
    let b = Q::<U4, U4>::from_bits(1); // 0.0625
    let c = a.wrapping_add(b); // Should wrap to MIN
    assert_eq!(c, Q::<U4, U4>::MIN);
}

/// Verifies wrapping_mul: 2.0 * 3.5 = 7.0 in Q8.8.
#[test]
fn q_wrapping_mul() {
    let a = Q::<U8, U8>::from(2i32);
    let b = Q::<U8, U8>::from_f64(3.5);
    let c = a.wrapping_mul(b);
    assert_eq!(c.to_f64(), 7.0);
}

/// Verifies wrapping_div: 7.0 / 2.0 = 3.5 in Q8.8.
#[test]
fn q_wrapping_div() {
    let a = Q::<U8, U8>::from(7i32);
    let b = Q::<U8, U8>::from(2i32);
    let c = a.wrapping_div(b);
    assert!((c.to_f64() - 3.5).abs() < 1e-10);
}

// ---------------------------------------------------------------------------
// Q: Widening multiply
// ---------------------------------------------------------------------------

/// Verifies widening_mul with integer operands: 3 * 4 = 12.
#[test]
fn q_widening_mul_basic() {
    let a = Q::<U12, U4>::from(3i32);
    let b = Q::<U12, U4>::from(4i32);
    let c: Q<U24, U8> = a.widening_mul(b);
    assert_eq!(c.to_f64(), 12.0);
}

/// Verifies widening_mul preserves fractional precision: 2.5 * 3.0 = 7.5.
#[test]
fn q_widening_mul_fractional() {
    let a = Q::<U12, U4>::from_f64(2.5);
    let b = Q::<U12, U4>::from_f64(3.0);
    let c: Q<U24, U8> = a.widening_mul(b);
    assert!((c.to_f64() - 7.5).abs() < 0.01);
}

/// Verifies widening_mul across different Q formats: Q12.4 * Q4.12.
#[test]
fn q_widening_mul_cross_type() {
    let a = Q::<U12, U4>::from(5i32);
    let b = Q::<U4, U12>::from_f64(0.5);
    let c: Q<U16, U16> = a.widening_mul(b);
    assert!((c.to_f64() - 2.5).abs() < 0.001);
}

// ---------------------------------------------------------------------------
// UQ: Arithmetic
// ---------------------------------------------------------------------------

/// Verifies UQ addition: 0.5 + 0.25 = 0.75.
#[test]
fn uq_add() {
    let a = UQ::<U1, U7>::from_f64(0.5);
    let b = UQ::<U1, U7>::from_f64(0.25);
    let c = a + b;
    assert!((c.to_f64() - 0.75).abs() < 0.01);
}

/// Verifies that UQ subtraction returns signed Q (can be negative).
#[test]
fn uq_sub_returns_signed() {
    let a = UQ::<U1, U7>::from_f64(0.25);
    let b = UQ::<U1, U7>::from_f64(0.5);
    let c: Q<U2, U7> = a - b;
    assert!((c.to_f64() - (-0.25)).abs() < 0.01);
}

// ---------------------------------------------------------------------------
// Shift operations
// ---------------------------------------------------------------------------

/// Verifies left shift: 1 << 3 = 8 in integer part.
#[test]
fn q_shift_left() {
    let a = Q::<U12, U4>::from(1i32);
    let b = a << 3;
    assert_eq!(b.to_f64(), 8.0);
}

/// Verifies arithmetic right shift preserves sign: -8 >> 2 = -2.
#[test]
fn q_arithmetic_shift_right() {
    let a = Q::<U12, U4>::from(-8i32);
    let b = a >> 2;
    assert_eq!(b.to_f64(), -2.0);
}

/// Verifies logical right shift fills with zero: 128 >> 1 = 64.
#[test]
fn uq_logical_shift_right() {
    let a = UQ::<U8, U0>::from(128u32);
    let b = a >> 1;
    assert_eq!(b.to_f64(), 64.0);
}

// ---------------------------------------------------------------------------
// Bitwise operations
// ---------------------------------------------------------------------------

/// Verifies bitwise AND between two Q8.0 values.
#[test]
fn q_bitand() {
    let a = Q::<U8, U0>::from(0b1010_1100u8 as i8 as i32);
    let b = Q::<U8, U0>::from(0b1111_0000u8 as i8 as i32);
    let c = a & b;
    assert_eq!(c.to_bits(), 0b1010_0000);
}

// ---------------------------------------------------------------------------
// min, max, clamp
// ---------------------------------------------------------------------------

/// Verifies min, max, and clamp behavior.
#[test]
fn q_min_max_clamp() {
    let a = Q::<U8, U8>::from(3i32);
    let b = Q::<U8, U8>::from(7i32);
    let lo = Q::<U8, U8>::from(4i32);
    let hi = Q::<U8, U8>::from(6i32);

    assert_eq!(a.min(b).to_f64(), 3.0);
    assert_eq!(a.max(b).to_f64(), 7.0);
    assert_eq!(a.clamp(lo, hi).to_f64(), 4.0); // 3 clamped up to 4
    assert_eq!(b.clamp(lo, hi).to_f64(), 6.0); // 7 clamped down to 6
}

// ---------------------------------------------------------------------------
// abs
// ---------------------------------------------------------------------------

/// Verifies abs returns magnitude for both negative and positive values.
#[test]
fn q_abs() {
    let a = Q::<U8, U8>::from(-5i32);
    assert_eq!(a.abs().to_f64(), 5.0);
    let b = Q::<U8, U8>::from(3i32);
    assert_eq!(b.abs().to_f64(), 3.0);
}

// ---------------------------------------------------------------------------
// Q: Saturating arithmetic
// ---------------------------------------------------------------------------

/// saturating_add clamps to MAX on positive overflow.
#[test]
fn q_saturating_add_overflow() {
    let max = Q::<U4, U4>::MAX;
    let one = Q::<U4, U4>::from(1i32);
    assert_eq!(max.saturating_add(one), Q::<U4, U4>::MAX);
}

/// saturating_add works normally when no overflow.
#[test]
fn q_saturating_add_normal() {
    let a = Q::<U8, U8>::from(3i32);
    let b = Q::<U8, U8>::from(4i32);
    assert_eq!(a.saturating_add(b).to_f64(), 7.0);
}

/// saturating_sub clamps to MIN on negative overflow.
#[test]
fn q_saturating_sub_underflow() {
    let min = Q::<U4, U4>::MIN;
    let one = Q::<U4, U4>::from(1i32);
    assert_eq!(min.saturating_sub(one), Q::<U4, U4>::MIN);
}

/// saturating_sub works normally when no underflow.
#[test]
fn q_saturating_sub_normal() {
    let a = Q::<U8, U8>::from(7i32);
    let b = Q::<U8, U8>::from(3i32);
    assert_eq!(a.saturating_sub(b).to_f64(), 4.0);
}

/// saturating_mul clamps to MAX on positive overflow.
#[test]
fn q_saturating_mul_overflow() {
    let max = Q::<U4, U4>::MAX;
    let two = Q::<U4, U4>::from(2i32);
    assert_eq!(max.saturating_mul(two), Q::<U4, U4>::MAX);
}

/// saturating_mul clamps to MIN on negative overflow.
#[test]
fn q_saturating_mul_negative_overflow() {
    let max = Q::<U4, U4>::MAX;
    let neg_two = Q::<U4, U4>::from(-2i32);
    assert_eq!(max.saturating_mul(neg_two), Q::<U4, U4>::MIN);
}

/// saturating_mul works normally when no overflow.
#[test]
fn q_saturating_mul_normal() {
    let a = Q::<U8, U8>::from_f64(2.0);
    let b = Q::<U8, U8>::from_f64(3.5);
    assert_eq!(a.saturating_mul(b).to_f64(), 7.0);
}

/// saturating_neg: MIN saturates to MAX.
#[test]
fn q_saturating_neg_min() {
    let min = Q::<U4, U4>::MIN;
    assert_eq!(min.saturating_neg(), Q::<U4, U4>::MAX);
}

/// saturating_neg works normally for non-MIN values.
#[test]
fn q_saturating_neg_normal() {
    let a = Q::<U8, U8>::from(5i32);
    assert_eq!(a.saturating_neg().to_f64(), -5.0);
    let b = Q::<U8, U8>::from(-3i32);
    assert_eq!(b.saturating_neg().to_f64(), 3.0);
}

// ---------------------------------------------------------------------------
// UQ: Saturating arithmetic
// ---------------------------------------------------------------------------

/// saturating_add clamps to MAX on overflow.
#[test]
fn uq_saturating_add_overflow() {
    let max = UQ::<U4, U4>::MAX;
    let one = UQ::<U4, U4>::from(1u32);
    assert_eq!(max.saturating_add(one), UQ::<U4, U4>::MAX);
}

/// saturating_add works normally when no overflow.
#[test]
fn uq_saturating_add_normal() {
    let a = UQ::<U8, U8>::from(3u32);
    let b = UQ::<U8, U8>::from(4u32);
    assert_eq!(a.saturating_add(b).to_f64(), 7.0);
}

/// saturating_sub clamps to zero on underflow.
#[test]
fn uq_saturating_sub_underflow() {
    let a = UQ::<U4, U4>::from(1u32);
    let b = UQ::<U4, U4>::from(3u32);
    assert_eq!(a.saturating_sub(b), UQ::<U4, U4>::ZERO);
}

/// saturating_sub works normally when no underflow.
#[test]
fn uq_saturating_sub_normal() {
    let a = UQ::<U8, U8>::from(7u32);
    let b = UQ::<U8, U8>::from(3u32);
    assert_eq!(a.saturating_sub(b).to_f64(), 4.0);
}

/// saturating_mul clamps to MAX on overflow.
#[test]
fn uq_saturating_mul_overflow() {
    let max = UQ::<U4, U4>::MAX;
    let two = UQ::<U4, U4>::from(2u32);
    assert_eq!(max.saturating_mul(two), UQ::<U4, U4>::MAX);
}

/// saturating_mul works normally when no overflow.
#[test]
fn uq_saturating_mul_normal() {
    let a = UQ::<U8, U8>::from_f64(2.0);
    let b = UQ::<U8, U8>::from_f64(3.5);
    assert_eq!(a.saturating_mul(b).to_f64(), 7.0);
}

// ---------------------------------------------------------------------------
// Q: Widening add/sub
// ---------------------------------------------------------------------------

/// widening_add: MAX + MAX fits in wider output.
#[test]
fn q_widening_add_no_overflow() {
    let max = Q::<U4, U4>::MAX; // 7.9375
    let sum: Q<U5, U4> = max.widening_add(max);
    assert_eq!(sum.to_f64(), max.to_f64() * 2.0);
}

/// widening_add: normal values with same fractional bits.
#[test]
fn q_widening_add_normal() {
    let a = Q::<U4, U4>::from_f64(2.5);
    let b = Q::<U4, U4>::from_f64(3.25);
    let sum: Q<U5, U4> = a.widening_add(b);
    assert_eq!(sum.to_f64(), 5.75);
}

/// widening_add: fractional alignment when FO > F.
#[test]
fn q_widening_add_frac_widen() {
    let a = Q::<U4, U4>::from_f64(1.5);
    let b = Q::<U4, U4>::from_f64(2.25);
    let sum: Q<U5, U8> = a.widening_add(b);
    assert_eq!(sum.to_f64(), 3.75);
}

/// widening_sub: MIN - MAX fits in wider output.
#[test]
fn q_widening_sub_no_overflow() {
    let min = Q::<U4, U4>::MIN; // -8.0
    let max = Q::<U4, U4>::MAX; // 7.9375
    let diff: Q<U5, U4> = min.widening_sub(max);
    assert_eq!(diff.to_f64(), min.to_f64() - max.to_f64());
}

/// widening_sub: normal case.
#[test]
fn q_widening_sub_normal() {
    let a = Q::<U4, U4>::from_f64(1.5);
    let b = Q::<U4, U4>::from_f64(3.75);
    let diff: Q<U5, U4> = a.widening_sub(b);
    assert_eq!(diff.to_f64(), -2.25);
}

// ---------------------------------------------------------------------------
// UQ: Widening add/sub
// ---------------------------------------------------------------------------

/// widening_add: MAX + MAX fits in wider output.
#[test]
fn uq_widening_add_no_overflow() {
    let max = UQ::<U4, U4>::MAX; // 15.9375
    let sum: UQ<U5, U4> = max.widening_add(max);
    assert_eq!(sum.to_f64(), max.to_f64() * 2.0);
}

/// widening_add: normal case.
#[test]
fn uq_widening_add_normal() {
    let a = UQ::<U4, U4>::from_f64(5.5);
    let b = UQ::<U4, U4>::from_f64(3.25);
    let sum: UQ<U5, U4> = a.widening_add(b);
    assert_eq!(sum.to_f64(), 8.75);
}

/// widening_sub: returns signed Q, positive result.
#[test]
fn uq_widening_sub_positive() {
    let a = UQ::<U4, U4>::from_f64(5.5);
    let b = UQ::<U4, U4>::from_f64(3.25);
    let diff: Q<U5, U4> = a.widening_sub(b);
    assert_eq!(diff.to_f64(), 2.25);
}

/// widening_sub: returns signed Q, negative result.
#[test]
fn uq_widening_sub_negative() {
    let a = UQ::<U4, U4>::from_f64(2.0);
    let b = UQ::<U4, U4>::from_f64(5.5);
    let diff: Q<U5, U4> = a.widening_sub(b);
    assert_eq!(diff.to_f64(), -3.5);
}

/// widening_sub: fractional alignment when FO > F.
#[test]
fn uq_widening_sub_frac_widen() {
    let a = UQ::<U4, U4>::from_f64(3.0);
    let b = UQ::<U4, U4>::from_f64(1.5);
    let diff: Q<U5, U8> = a.widening_sub(b);
    assert_eq!(diff.to_f64(), 1.5);
}

// ---------------------------------------------------------------------------
// Sum: logarithmic bit growth into a caller-chosen accumulator
// ---------------------------------------------------------------------------

/// Summing 64 `Q<U12, U4>` values fits in `Q<U18, U4>` (only 6 extra integer
/// bits, not 64) without overflow.
#[test]
fn q_sum_64_values_six_bits_growth() {
    let xs = [Q::<U12, U4>::from(1000i32); 64];
    let total: Q<U18, U4> = xs.iter().copied().sum();
    assert_eq!(total.to_f64(), 64_000.0);
}

/// `Sum` works over owned values and preserves fractional precision.
#[test]
fn q_sum_owned_fractional() {
    let xs = [
        Q::<U8, U8>::from_f64(0.5),
        Q::<U8, U8>::from_f64(0.25),
        Q::<U8, U8>::from_f64(1.25),
    ];
    let total: Q<U10, U8> = xs.into_iter().sum();
    assert!((total.to_f64() - 2.0).abs() < 1e-6);
}

/// `Sum` over a borrowing iterator (`&Q`) matches the owned result.
#[test]
fn q_sum_borrowed() {
    let xs = [Q::<U8, U8>::from(3i32), Q::<U8, U8>::from(4i32)];
    let total: Q<U10, U8> = xs.iter().sum();
    assert_eq!(total.to_f64(), 7.0);
}

/// `Sum` handles negative values; the wide accumulator never overflows.
#[test]
fn q_sum_negative_values() {
    let xs = [Q::<U4, U4>::MIN; 8]; // 8 * -8.0 = -64.0
    let total: Q<U8, U4> = xs.iter().copied().sum();
    assert_eq!(total.to_f64(), -64.0);
}

/// `Sum` of an empty iterator yields ZERO.
#[test]
fn q_sum_empty_is_zero() {
    let total: Q<U18, U4> = core::iter::empty::<Q<U12, U4>>().sum();
    assert_eq!(total, Q::<U18, U4>::ZERO);
}

/// `Sum` realigns the fractional point when `FO != F`.
#[test]
fn q_sum_frac_realign() {
    let xs = [Q::<U8, U4>::from_f64(1.5), Q::<U8, U4>::from_f64(2.25)];
    let total: Q<U10, U8> = xs.iter().copied().sum();
    assert_eq!(total.to_f64(), 3.75);
}

/// UQ `Sum` grows logarithmically and stays unsigned.
#[test]
fn uq_sum_64_values() {
    let xs = [UQ::<U8, U8>::from(200u32); 64];
    let total: UQ<U14, U8> = xs.iter().copied().sum();
    assert_eq!(total.to_f64(), 12_800.0);
}

/// UQ `Sum` over borrowed values, with fractional precision.
#[test]
fn uq_sum_borrowed_fractional() {
    let xs = [
        UQ::<U1, U7>::from_f64(0.5),
        UQ::<U1, U7>::from_f64(0.25),
        UQ::<U1, U7>::from_f64(0.125),
    ];
    let total: UQ<U3, U7> = xs.iter().sum();
    assert!((total.to_f64() - 0.875).abs() < 1e-3);
}

// ---------------------------------------------------------------------------
// try_sum: checked accumulation (errors instead of wrapping)
// ---------------------------------------------------------------------------

/// `try_sum` succeeds when the accumulator is wide enough for all N values.
#[test]
fn q_try_sum_ok() {
    let xs = [Q::<U12, U4>::from(1000i32); 64];
    let total = Q::<U18, U4>::try_sum(xs).unwrap();
    assert_eq!(total.to_f64(), 64_000.0);
}

/// `try_sum` errors when the chosen output type is too narrow to hold the sum,
/// rather than silently wrapping like the `Sum` impl.
#[test]
fn q_try_sum_too_narrow_errors() {
    let xs = [Q::<U12, U4>::from(1000i32); 64];
    assert_eq!(
        Q::<U12, U4>::try_sum(xs),
        Err(qfixed::FixedError::OutOfRange)
    );
}

/// `try_sum` over an empty iterator is `Ok(ZERO)`.
#[test]
fn q_try_sum_empty_ok_zero() {
    let total = Q::<U18, U4>::try_sum(core::iter::empty::<Q<U12, U4>>()).unwrap();
    assert_eq!(total, Q::<U18, U4>::ZERO);
}

/// `try_sum` accepts borrowing iterators and detects negative overflow.
#[test]
fn q_try_sum_negative_overflow_errors() {
    let xs = [Q::<U4, U4>::MIN; 8]; // 8 * -8.0 = -64.0, needs |I| >= 8
    assert!(Q::<U6, U4>::try_sum(xs.iter().copied()).is_err());
    assert_eq!(Q::<U8, U4>::try_sum(xs).unwrap().to_f64(), -64.0);
}

/// UQ `try_sum` succeeds within range and errors on overflow.
#[test]
fn uq_try_sum_ok_and_overflow() {
    let xs = [UQ::<U8, U8>::from(200u32); 64]; // 12_800 needs 14 integer bits
    assert_eq!(UQ::<U14, U8>::try_sum(xs).unwrap().to_f64(), 12_800.0);
    assert!(UQ::<U8, U8>::try_sum(xs).is_err());
}
