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

/// Verifies a negative value's bit pattern is stored and extracted correctly.
#[test]
fn q_negative_from_bits() {
    // Q4.4: 8-bit signed, value -1.0 has bit pattern 0xF0 (count -16).
    let v = Q::<U4, U4>::from_bits(0xF0);
    assert_eq!(v.to_bits(), 0xF0);
    assert_eq!(v.to_count(), -16);
}

/// Verifies sign extension works for all-bits-set patterns.
#[test]
fn q_sign_extension() {
    // Q4.4: all bits set (0xFF) = -0.0625.
    let v = Q::<U4, U4>::from_bits(0xFF);
    assert_eq!(v.to_bits(), 0xFF);
    assert_eq!(v.to_count(), -1);
}

/// Verifies `from_bits(to_bits(x)) == x` for signed values of both signs.
///
/// `to_bits` returns the unsigned bit pattern, so for negative values it reads
/// back as a large positive; `from_bits` must invert that. Values are built
/// with `from_count` (the signed LSB-count view).
#[test]
fn q_bits_roundtrip_signed_values() {
    // Q8.8 is 16-bit: count range [-32768, 32767].
    for &count in &[-32768i64, -1000, -256, -16, -1, 0, 1, 16, 256, 1000, 32767] {
        let x = Q::<U8, U8>::from_count(count);
        assert_eq!(
            Q::<U8, U8>::from_bits(x.to_bits()),
            x,
            "round-trip failed for count {count}"
        );
    }
}

/// Verifies the unsigned `from_bits` and the signed `from_count` agree.
#[test]
fn q_from_bits_matches_from_count() {
    // Q4.4: -1.0 is count -16, whose 8-bit pattern is 0xF0.
    assert_eq!(Q::<U4, U4>::from_bits(0xF0), Q::<U4, U4>::from_count(-16));
    assert_eq!(Q::<U4, U4>::from_bits(0xF0).to_count(), -16);
}

/// Verifies the bit round-trip at full 64-bit width (no masking path).
#[test]
fn q_bits_roundtrip_full_width() {
    let x = Q::<U32, U32>::from(-5i32);
    assert_eq!(Q::<U32, U32>::from_bits(x.to_bits()), x);
}

/// Verifies whole-number construction via `From` and the LSB-count view.
#[test]
fn q_from_whole_number() {
    let v = Q::<U12, U4>::from(42i32);
    assert_eq!(v.to_f64(), 42.0);
    assert_eq!(v.to_count(), 42 << 4); // 42 units of 1/16
    assert_eq!(v.to_bits(), 42u64 << 4);
}

/// Verifies `from_count` builds a fractional value from a count of LSB units.
#[test]
fn q_from_count_fractional() {
    // Q4.4: one count is 1/16, so 56 counts = 3.5.
    let v = Q::<U4, U4>::from_count(56);
    assert_eq!(v.to_f64(), 3.5);
    assert_eq!(v.to_count(), 56);
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
    assert_eq!(Q::<U12, U4>::MIN.to_f64(), -2048.0); // -2^(I-1)
}

// ---------------------------------------------------------------------------
// Q: Non-power-of-2 bit widths (the key motivation)
// ---------------------------------------------------------------------------

/// Verifies Q11.0 (11-bit signed integer) for edge function A/B
/// coefficients from the RTL rasterizer.
#[test]
fn q_11bit_edge_coeff() {
    let a = Q::<U11, U0>::from(500i32);
    let b = Q::<U11, U0>::from(-300i32);
    assert_eq!(a.to_count(), 500); // F = 0, so count == integer value
    assert_eq!(b.to_count(), -300);
    assert_eq!((a + b).to_count(), 200);

    assert_eq!(Q::<U11, U0>::MAX.to_count(), 1023);
    assert_eq!(Q::<U11, U0>::MIN.to_count(), -1024);
}

/// Verifies Q21.0 (21-bit signed integer) for edge function C
/// coefficients.
#[test]
fn q_21bit_edge_constant() {
    let a = Q::<U21, U0>::from(100_000i32);
    let b = Q::<U21, U0>::from(-50_000i32);
    assert_eq!((a + b).to_count(), 50_000);
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

/// Verifies `from_bits(to_bits(x)) == x` for UQ values, including the
/// high-bit-set patterns that exercise the masking path.
#[test]
fn uq_bits_roundtrip_values() {
    // UQ4.4 is 8-bit: count range [0, 255].
    for &count in &[0u64, 1, 0x0F, 0x80, 0xC8, 0xFF] {
        let x = UQ::<U4, U4>::from_count(count);
        assert_eq!(
            UQ::<U4, U4>::from_bits(x.to_bits()),
            x,
            "round-trip failed for count {count}"
        );
    }
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
    let a = Q::<U32, U32>::from(1i32);
    let b = Q::<U32, U32>::from(2i32);
    let c = a.wrapping_add(b);
    assert_eq!(c.to_f64(), 3.0);
}
