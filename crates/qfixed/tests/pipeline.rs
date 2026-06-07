//! Realistic Q4.12 pipeline tests (the primary digital-twin format).

use qfixed::Q;
use qfixed::typenum::consts::*;

/// Saturating add on Q4.12 (the main DT format).
#[test]
fn q4_12_saturating_add() {
    let a = Q::<U4, U12>::from_f64(0.75);
    let b = Q::<U4, U12>::from_f64(0.5);
    let sum = a.saturating_add(b);
    // 1.25 fits in Q4.12 (range [-8, ~8)
    assert!((sum.to_f64() - 1.25).abs() < 1e-4);
}

/// Saturating mul on Q4.12 (color blending).
#[test]
fn q4_12_saturating_mul() {
    let color = Q::<U4, U12>::from_f64(0.8);
    let alpha = Q::<U4, U12>::from_f64(0.5);
    let result = color.saturating_mul(alpha);
    assert!((result.to_f64() - 0.4).abs() < 1e-3);
}

/// Widening mul then saturate back (typical pipeline pattern).
#[test]
fn q4_12_widening_mul_then_saturate() {
    let a = Q::<U4, U12>::from_f64(1.5);
    let b = Q::<U4, U12>::from_f64(2.0);
    let wide: Q<U8, U24> = a.widening_mul(b);
    let narrow: Q<U4, U12> = wide.saturate();
    assert!((narrow.to_f64() - 3.0).abs() < 1e-3);
}
