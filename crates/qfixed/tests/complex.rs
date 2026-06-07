//! Tests for the signed complex fixed-point type `CQ<I, F>`: construction,
//! arithmetic (operators, wrapping, saturating, conjugate, rotation,
//! magnitude), conversions, shifts, and formatting.

use qfixed::typenum::consts::*;
use qfixed::{CQ, Q};

/// Tolerance for Q8.8 round-trip comparisons (one LSB is 1/256).
const EPS: f64 = 1e-3;

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

/// Verifies `new` stores the real and imaginary components.
#[test]
fn cq_new() {
    let z = CQ::<U8, U8>::new(Q::from_f64(1.5), Q::from_f64(-2.25));
    assert!((z.re.to_f64() - 1.5).abs() < EPS);
    assert!((z.im.to_f64() - (-2.25)).abs() < EPS);
}

/// Verifies `from_f64` quantizes both components.
#[test]
fn cq_from_f64() {
    let z = CQ::<U8, U8>::from_f64(3.0, 4.0);
    assert!((z.re.to_f64() - 3.0).abs() < EPS);
    assert!((z.im.to_f64() - 4.0).abs() < EPS);
}

/// Verifies `from_re` zeroes the imaginary part.
#[test]
fn cq_from_re() {
    let z = CQ::<U8, U8>::from_re(Q::from(5i32));
    assert_eq!(z.re.to_f64(), 5.0);
    assert_eq!(z.im, Q::<U8, U8>::ZERO);
}

/// Verifies the `ZERO`, `ONE`, and `J` constants.
#[test]
fn cq_constants() {
    assert_eq!(CQ::<U8, U8>::ZERO, CQ::new(Q::ZERO, Q::ZERO));
    assert_eq!(CQ::<U8, U8>::ONE, CQ::new(Q::ONE, Q::ZERO));
    assert_eq!(CQ::<U8, U8>::J, CQ::new(Q::ZERO, Q::ONE));
}

/// Verifies `from_bits` / `to_bits` round-trip, including negative components
/// whose masked words read back as large positives.
#[test]
fn cq_bits_roundtrip() {
    for &(re, im) in &[(1.5, -2.25), (-3.0, -4.0), (-1.0, 2.0), (0.0, 0.0)] {
        let z = CQ::<U8, U8>::from_f64(re, im);
        let (re_bits, im_bits) = z.to_bits();
        assert_eq!(
            CQ::<U8, U8>::from_bits(re_bits, im_bits),
            z,
            "round-trip failed for ({re}, {im})"
        );
    }
}

// ---------------------------------------------------------------------------
// Arithmetic operators (widening)
// ---------------------------------------------------------------------------

/// Verifies complex addition widens by one integer bit.
#[test]
fn cq_add() {
    let a = CQ::<U8, U8>::from_f64(1.0, 2.0);
    let b = CQ::<U8, U8>::from_f64(3.0, 4.0);
    let c: CQ<U9, U8> = a + b;
    assert!((c.re.to_f64() - 4.0).abs() < EPS);
    assert!((c.im.to_f64() - 6.0).abs() < EPS);
}

/// Verifies complex subtraction.
#[test]
fn cq_sub() {
    let a = CQ::<U8, U8>::from_f64(3.0, 4.0);
    let b = CQ::<U8, U8>::from_f64(1.0, 2.0);
    let c: CQ<U9, U8> = a - b;
    assert!((c.re.to_f64() - 2.0).abs() < EPS);
    assert!((c.im.to_f64() - 2.0).abs() < EPS);
}

/// Verifies complex negation.
#[test]
fn cq_neg() {
    let a = CQ::<U8, U8>::from_f64(1.5, -2.0);
    let c: CQ<U9, U8> = -a;
    assert!((c.re.to_f64() - (-1.5)).abs() < EPS);
    assert!((c.im.to_f64() - 2.0).abs() < EPS);
}

/// Verifies complex multiply: `(1 + 2i)(3 + 4i) = -5 + 10i`, with the extra
/// integer bit (`Q8.8 * Q8.8` → `CQ<U17, U16>`).
#[test]
fn cq_mul() {
    let a = CQ::<U8, U8>::from_f64(1.0, 2.0);
    let b = CQ::<U8, U8>::from_f64(3.0, 4.0);
    let c: CQ<U17, U16> = a * b;
    assert!((c.re.to_f64() - (-5.0)).abs() < EPS);
    assert!((c.im.to_f64() - 10.0).abs() < EPS);
}

/// Verifies scalar multiply `CQ * Q` does not need the extra integer bit
/// (`Q8.8` scalar → `CQ<U16, U16>`).
#[test]
fn cq_scalar_mul_right() {
    let a = CQ::<U8, U8>::from_f64(1.0, 2.0);
    let s = Q::<U8, U8>::from(2i32);
    let c: CQ<U16, U16> = a * s;
    assert!((c.re.to_f64() - 2.0).abs() < EPS);
    assert!((c.im.to_f64() - 4.0).abs() < EPS);
}

/// Verifies scalar multiply `Q * CQ` from the left.
#[test]
fn cq_scalar_mul_left() {
    let a = CQ::<U8, U8>::from_f64(1.0, 2.0);
    let s = Q::<U8, U8>::from(3i32);
    let c: CQ<U16, U16> = s * a;
    assert!((c.re.to_f64() - 3.0).abs() < EPS);
    assert!((c.im.to_f64() - 6.0).abs() < EPS);
}

// ---------------------------------------------------------------------------
// Conjugate, rotation, magnitude
// ---------------------------------------------------------------------------

/// Verifies the widening conjugate flips the sign of the imaginary part.
#[test]
fn cq_conj() {
    let a = CQ::<U8, U8>::from_f64(1.0, 2.0);
    let c: CQ<U9, U8> = a.conj();
    assert!((c.re.to_f64() - 1.0).abs() < EPS);
    assert!((c.im.to_f64() - (-2.0)).abs() < EPS);
}

/// Verifies same-width `wrapping_conj`.
#[test]
fn cq_wrapping_conj() {
    let a = CQ::<U8, U8>::from_f64(1.0, 2.0);
    let c = a.wrapping_conj();
    assert!((c.re.to_f64() - 1.0).abs() < EPS);
    assert!((c.im.to_f64() - (-2.0)).abs() < EPS);
}

/// Verifies `mul_j` rotates by +90°: `(1 + 2i)i = -2 + i`.
#[test]
fn cq_mul_j() {
    let a = CQ::<U8, U8>::from_f64(1.0, 2.0);
    let c: CQ<U9, U8> = a.mul_j();
    assert!((c.re.to_f64() - (-2.0)).abs() < EPS);
    assert!((c.im.to_f64() - 1.0).abs() < EPS);
}

/// Verifies `mul_neg_j` rotates by -90°: `(1 + 2i)(-i) = 2 - i`.
#[test]
fn cq_mul_neg_j() {
    let a = CQ::<U8, U8>::from_f64(1.0, 2.0);
    let c: CQ<U9, U8> = a.mul_neg_j();
    assert!((c.re.to_f64() - 2.0).abs() < EPS);
    assert!((c.im.to_f64() - (-1.0)).abs() < EPS);
}

/// Verifies `norm_sqr`: `|3 + 4i|^2 = 25` (`Q8.8` → real `Q<U17, U16>`).
#[test]
fn cq_norm_sqr() {
    let a = CQ::<U8, U8>::from_f64(3.0, 4.0);
    let n: Q<U17, U16> = a.norm_sqr();
    assert!((n.to_f64() - 25.0).abs() < EPS);
}

// ---------------------------------------------------------------------------
// Same-width wrapping / saturating arithmetic
// ---------------------------------------------------------------------------

/// Verifies componentwise `wrapping_add` wraps the real part at `MAX`.
#[test]
fn cq_wrapping_add_wraps() {
    let a = CQ::<U4, U4>::new(Q::MAX, Q::ZERO);
    let b = CQ::<U4, U4>::new(Q::from_bits(1), Q::ZERO);
    let c = a.wrapping_add(b);
    assert_eq!(c.re, Q::<U4, U4>::MIN);
}

/// Verifies same-width complex `wrapping_mul`: `(1 + 2i)(3 + 4i) = -5 + 10i`
/// fits in `Q8.8`.
#[test]
fn cq_wrapping_mul() {
    let a = CQ::<U8, U8>::from_f64(1.0, 2.0);
    let b = CQ::<U8, U8>::from_f64(3.0, 4.0);
    let c = a.wrapping_mul(b);
    assert!((c.re.to_f64() - (-5.0)).abs() < EPS);
    assert!((c.im.to_f64() - 10.0).abs() < EPS);
}

/// Verifies `saturating_mul` clamps the overflowing component to `MAX`.
///
/// `(10 + 10i)^2 = 0 + 200i`; 200 exceeds the `Q8.8` range and saturates.
#[test]
fn cq_saturating_mul_clamps() {
    let a = CQ::<U8, U8>::from_f64(10.0, 10.0);
    let c = a.saturating_mul(a);
    assert!(c.re.to_f64().abs() < EPS);
    assert_eq!(c.im, Q::<U8, U8>::MAX);
}

/// Verifies `saturating_add` works normally when no overflow occurs.
#[test]
fn cq_saturating_add_normal() {
    let a = CQ::<U8, U8>::from_f64(1.0, 2.0);
    let b = CQ::<U8, U8>::from_f64(3.0, 4.0);
    let c = a.saturating_add(b);
    assert!((c.re.to_f64() - 4.0).abs() < EPS);
    assert!((c.im.to_f64() - 6.0).abs() < EPS);
}

// ---------------------------------------------------------------------------
// Conversions
// ---------------------------------------------------------------------------

/// Verifies lossless widening preserves both components.
#[test]
fn cq_widen() {
    let a = CQ::<U8, U8>::from_f64(1.5, -2.5);
    let w: CQ<U16, U16> = a.widen();
    assert!((w.re.to_f64() - 1.5).abs() < EPS);
    assert!((w.im.to_f64() - (-2.5)).abs() < EPS);
}

/// Verifies narrowing with saturation clamps both components.
#[test]
fn cq_saturate() {
    let a = CQ::<U8, U8>::from_f64(100.0, -100.0);
    let s: CQ<U4, U8> = a.saturate();
    assert_eq!(s.re, Q::<U4, U8>::MAX);
    assert_eq!(s.im, Q::<U4, U8>::MIN);
}

/// Verifies narrowing with truncation keeps representable values.
#[test]
fn cq_truncate() {
    let a = CQ::<U8, U16>::from_f64(1.25, -1.25);
    let t: CQ<U8, U8> = a.truncate();
    assert!((t.re.to_f64() - 1.25).abs() < EPS);
    assert!((t.im.to_f64() - (-1.25)).abs() < EPS);
}

// ---------------------------------------------------------------------------
// Shifts
// ---------------------------------------------------------------------------

/// Verifies left shift scales both components.
#[test]
fn cq_shl() {
    let a = CQ::<U8, U8>::from_f64(1.0, 2.0);
    let c = a << 1;
    assert!((c.re.to_f64() - 2.0).abs() < EPS);
    assert!((c.im.to_f64() - 4.0).abs() < EPS);
}

/// Verifies arithmetic right shift preserves sign on both components.
#[test]
fn cq_shr() {
    let a = CQ::<U8, U8>::from_f64(2.0, -4.0);
    let c = a >> 1;
    assert!((c.re.to_f64() - 1.0).abs() < EPS);
    assert!((c.im.to_f64() - (-2.0)).abs() < EPS);
}

// ---------------------------------------------------------------------------
// Formatting
// ---------------------------------------------------------------------------

/// Verifies `Display` renders `re±imj` with a negative imaginary part.
#[test]
fn cq_display_negative_imag() {
    let z = CQ::<U8, U8>::from_f64(1.5, -2.25);
    assert_eq!(format!("{z}"), "1.5-2.25j");
}

/// Verifies `Display` renders an explicit `+` for a positive imaginary part.
#[test]
fn cq_display_positive_imag() {
    let z = CQ::<U8, U8>::from_f64(1.5, 2.25);
    assert_eq!(format!("{z}"), "1.5+2.25j");
}

/// Verifies `Debug` shows the CQ tag, raw register words, and value.
#[test]
fn cq_debug() {
    let z = CQ::<U8, U8>::from_f64(1.5, -2.25);
    assert_eq!(format!("{z:?}"), "CQ8.8(0x0180, 0xfdc0 = 1.5-2.25j)");
}

// ---------------------------------------------------------------------------
// Pipeline: a Q4.12 IQ-sample complex multiply (digital-down-conversion mixer)
// ---------------------------------------------------------------------------

/// A realistic IQ mixer step: multiply a sample by a unit-magnitude rotator,
/// then narrow back to the sample format. `(0.5 + 0.5i) * i = -0.5 + 0.5i`.
#[test]
fn cq_iq_mixer_pipeline() {
    let sample = CQ::<U4, U12>::from_f64(0.5, 0.5);
    let rotated: CQ<U5, U12> = sample.mul_j();
    let narrowed: CQ<U4, U12> = rotated.saturate();
    assert!((narrowed.re.to_f64() - (-0.5)).abs() < 1e-3);
    assert!((narrowed.im.to_f64() - 0.5).abs() < 1e-3);
}
