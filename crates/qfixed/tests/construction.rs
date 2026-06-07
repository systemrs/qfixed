//! Tests for `Q`/`UQ` construction, bit representation, and constants.

use qfixed::typenum::consts::*;
use qfixed::{Q, UQ};

// ---------------------------------------------------------------------------
// Q: Construction and bit representation
// ---------------------------------------------------------------------------

/// Verifies `from_bits` and `to_bits` are lossless inverses for Q12.4.
#[test]
fn q_from_bits_roundtrip() {
    let v = Q::<U12, U4>::from_bits(0x0108); // 16.5 in Q12.4
    assert_eq!(v.to_bits(), 0x0108);
}

/// Verifies negative values in Q4.4 are correctly stored and extracted.
#[test]
fn q_negative_from_bits() {
    // Q4.4: 8-bit signed, value -1.0 = 0xF0 in 8 bits
    let v = Q::<U4, U4>::from_bits(-16); // -1.0 in Q4.4 = -16 raw
    assert_eq!(v.to_bits(), 0xF0); // 8-bit representation
    assert_eq!(v.to_int(), -1);
}

/// Verifies sign extension works for all-bits-set values.
#[test]
fn q_sign_extension() {
    // Q4.4: -1 raw should sign-extend. Pass as sign-extended i64.
    let v = Q::<U4, U4>::from_bits(-1); // All bits set = -0.0625
    assert_eq!(v.to_bits(), 0xFF); // 8-bit representation: all bits set
}

/// Verifies `from_int` sets the integer part and `to_int` recovers it.
#[test]
fn q_from_int() {
    let v = Q::<U12, U4>::from_int(42);
    assert_eq!(v.to_int(), 42);
    assert_eq!(v.to_bits(), 42 << 4);
}

/// Verifies `from_f64` quantizes 16.5 to the correct Q12.4 bits.
#[test]
fn q_from_f64() {
    let v = Q::<U12, U4>::from_f64(16.5);
    assert_eq!(v.to_bits(), 0x108); // 16 * 16 + 8 = 264 = 0x108
}

/// Verifies `to_f64` recovers the original floating-point value.
#[test]
fn q_to_f64() {
    let v = Q::<U12, U4>::from_bits(0x108);
    assert!((v.to_f64() - 16.5).abs() < 1e-10);
}

/// Verifies ZERO, ONE, MAX, and MIN constants for Q12.4.
#[test]
fn q_constants() {
    assert_eq!(Q::<U12, U4>::ZERO.to_bits(), 0);
    assert_eq!(Q::<U12, U4>::ONE.to_bits(), 16); // 1.0 in Q12.4 = 16
    assert_eq!(Q::<U12, U4>::MAX.to_bits(), 0x7FFF); // 15-bit max
    assert_eq!(Q::<U12, U4>::MIN.to_int(), -(1i64 << 11)); // -2048
}

// ---------------------------------------------------------------------------
// Q: Non-power-of-2 bit widths (the key motivation)
// ---------------------------------------------------------------------------

/// Verifies Q11.0 (11-bit signed integer) for edge function A/B
/// coefficients from the RTL rasterizer.
#[test]
fn q_11bit_edge_coeff() {
    let a = Q::<U11, U0>::from_int(500);
    let b = Q::<U11, U0>::from_int(-300);
    assert_eq!(a.to_int(), 500);
    assert_eq!(b.to_int(), -300);
    assert_eq!((a + b).to_int(), 200);

    assert_eq!(Q::<U11, U0>::MAX.to_int(), 1023);
    assert_eq!(Q::<U11, U0>::MIN.to_int(), -1024);
}

/// Verifies Q21.0 (21-bit signed integer) for edge function C
/// coefficients.
#[test]
fn q_21bit_edge_constant() {
    let a = Q::<U21, U0>::from_int(100_000);
    let b = Q::<U21, U0>::from_int(-50_000);
    assert_eq!((a + b).to_int(), 50_000);
}

// ---------------------------------------------------------------------------
// UQ: Construction and constants
// ---------------------------------------------------------------------------

/// Verifies `from_bits` / `to_bits` roundtrip for UQ4.14 (18-bit).
#[test]
fn uq_from_bits_roundtrip() {
    let v = UQ::<U4, U14>::from_bits(0x1234);
    assert_eq!(v.to_bits(), 0x1234);
}

/// Verifies ZERO, ONE, and MAX constants for UQ1.7.
#[test]
fn uq_constants() {
    assert_eq!(UQ::<U1, U7>::ZERO.to_bits(), 0);
    assert_eq!(UQ::<U1, U7>::ONE.to_bits(), 128); // 1.0 in UQ1.7 = 128
    assert_eq!(UQ::<U1, U7>::MAX.to_bits(), 0xFF); // 8-bit max
}

/// Verifies UQ4.14 (18-bit) models the RTL 1/Q reciprocal format.
#[test]
fn uq_18bit_reciprocal() {
    let v = UQ::<U4, U14>::from_f64(0.5);
    assert!((v.to_f64() - 0.5).abs() < 0.001);
    assert_eq!(UQ::<U4, U14>::TOTAL_BITS, 18);
}

// ---------------------------------------------------------------------------
// Edge case: Q<U32, U32> (full 64-bit)
// ---------------------------------------------------------------------------

/// Verifies full 64-bit Q<U32, U32> wrapping addition works.
///
/// Q<U32, U32> is already at the 64-bit limit, so widening `+` can't be used.
/// Use `wrapping_add` for same-type addition at maximum width.
#[test]
fn q_64bit_full() {
    let a = Q::<U32, U32>::from_int(1);
    let b = Q::<U32, U32>::from_int(2);
    let c = a.wrapping_add(b);
    assert_eq!(c.to_int(), 3);
}
