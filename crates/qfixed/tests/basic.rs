//! Basic tests for qfixed `Q` and `UQ` types.

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
// Q: Arithmetic (std::ops)
// ---------------------------------------------------------------------------

/// Verifies Q addition: 3 + 4 = 7.
#[test]
fn q_add() {
    let a = Q::<U12, U4>::from_int(3);
    let b = Q::<U12, U4>::from_int(4);
    let c = a + b;
    assert_eq!(c.to_int(), 7);
}

/// Verifies Q subtraction: 10 - 3 = 7.
#[test]
fn q_sub() {
    let a = Q::<U12, U4>::from_int(10);
    let b = Q::<U12, U4>::from_int(3);
    let c = a - b;
    assert_eq!(c.to_int(), 7);
}

/// Verifies Q negation: -5 = -(5).
#[test]
fn q_neg() {
    let a = Q::<U12, U4>::from_int(5);
    let b = -a;
    assert_eq!(b.to_int(), -5);
}

/// Verifies that Q4.4 addition widens to Q5.4 (no overflow possible).
#[test]
fn q_add_widens() {
    let a = Q::<U4, U4>::MAX; // 7.9375
    let b = Q::<U4, U4>::from_int(1);
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
    let a = Q::<U8, U8>::from_int(2);
    let b = Q::<U8, U8>::from_f64(3.5);
    let c = a.wrapping_mul(b);
    assert_eq!(c.to_int(), 7);
}

/// Verifies wrapping_div: 7.0 / 2.0 = 3.5 in Q8.8.
#[test]
fn q_wrapping_div() {
    let a = Q::<U8, U8>::from_int(7);
    let b = Q::<U8, U8>::from_int(2);
    let c = a.wrapping_div(b);
    assert!((c.to_f64() - 3.5).abs() < 1e-10);
}

// ---------------------------------------------------------------------------
// Q: Widening multiply
// ---------------------------------------------------------------------------

/// Verifies widening_mul with integer operands: 3 * 4 = 12.
#[test]
fn q_widening_mul_basic() {
    let a = Q::<U12, U4>::from_int(3);
    let b = Q::<U12, U4>::from_int(4);
    let c: Q<U24, U8> = a.widening_mul(b);
    assert_eq!(c.to_int(), 12);
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
    let a = Q::<U12, U4>::from_int(5);
    let b = Q::<U4, U12>::from_f64(0.5);
    let c: Q<U16, U16> = a.widening_mul(b);
    assert!((c.to_f64() - 2.5).abs() < 0.001);
}

// ---------------------------------------------------------------------------
// Q: Conversions
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
// UQ: Construction and arithmetic
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
// UQ: Conversions
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
// Shift operations
// ---------------------------------------------------------------------------

/// Verifies left shift: 1 << 3 = 8 in integer part.
#[test]
fn q_shift_left() {
    let a = Q::<U12, U4>::from_int(1);
    let b = a << 3;
    assert_eq!(b.to_int(), 8);
}

/// Verifies arithmetic right shift preserves sign: -8 >> 2 = -2.
#[test]
fn q_arithmetic_shift_right() {
    let a = Q::<U12, U4>::from_int(-8);
    let b = a >> 2;
    assert_eq!(b.to_int(), -2);
}

/// Verifies logical right shift fills with zero: 128 >> 1 = 64.
#[test]
fn uq_logical_shift_right() {
    let a = UQ::<U8, U0>::from_int(128);
    let b = a >> 1;
    assert_eq!(b.to_int(), 64);
}

// ---------------------------------------------------------------------------
// Bitwise operations
// ---------------------------------------------------------------------------

/// Verifies bitwise AND between two Q8.0 values.
#[test]
fn q_bitand() {
    let a = Q::<U8, U0>::from_int(0b1010_1100u8 as i8 as i64);
    let b = Q::<U8, U0>::from_int(0b1111_0000u8 as i8 as i64);
    let c = a & b;
    assert_eq!(c.to_bits(), 0b1010_0000);
}

// ---------------------------------------------------------------------------
// Display and Debug formatting
// ---------------------------------------------------------------------------

/// Verifies Debug output starts with `Q12.4(` and contains the value.
#[test]
fn q_debug_format() {
    let v = Q::<U12, U4>::from_f64(16.5);
    let s = format!("{v:?}");
    assert!(s.starts_with("Q12.4("));
    assert!(s.contains("16.5"));
}

/// Verifies Display output contains the decimal value.
#[test]
fn q_display_format() {
    let v = Q::<U12, U4>::from_f64(16.5);
    let s = format!("{v}");
    assert!(s.contains("16.5"));
}

/// Verifies UQ Debug output starts with `UQ1.7(`.
#[test]
fn uq_debug_format() {
    let v = UQ::<U1, U7>::from_f64(0.5);
    let s = format!("{v:?}");
    assert!(s.starts_with("UQ1.7("));
}

/// Display honors an explicit precision from the format spec (the previous
/// hard-coded `.4` ignored it).
#[test]
fn q_display_honors_precision() {
    let v = Q::<U12, U4>::from_f64(16.5);
    assert_eq!(format!("{v:.2}"), "16.50");
    assert_eq!(format!("{v:.1}"), "16.5");
}

/// Display honors width and alignment (delegated to `f64`).
#[test]
fn q_display_honors_width() {
    let v = Q::<U12, U4>::from_f64(16.5);
    assert_eq!(format!("{v:>8}"), "    16.5");
}

/// With no precision, Display prints the shortest round-tripping decimal,
/// not trailing-zero padding.
#[test]
fn q_display_default_is_shortest() {
    assert_eq!(format!("{}", Q::<U12, U4>::from_f64(16.5)), "16.5");
    assert_eq!(format!("{}", Q::<U12, U4>::from_f64(0.0625)), "0.0625");
}

/// `{:x}`/`{:X}` print the raw register bits, zero-padded to the type width;
/// `#` adds the `0x` prefix.
#[test]
fn q_hex_is_raw_bits() {
    let v = Q::<U12, U4>::from_f64(16.5); // 0x108 in a 16-bit type
    assert_eq!(format!("{v:x}"), "0108");
    assert_eq!(format!("{v:X}"), "0108");
    assert_eq!(format!("{v:#x}"), "0x0108");
    assert_eq!(format!("{v:#X}"), "0x0108");
}

/// Negative values print as the unsigned two's-complement pattern, never with
/// a minus sign.
#[test]
fn q_hex_negative_is_twos_complement() {
    let v = Q::<U12, U4>::from_f64(-16.5); // -264 -> 0xFEF8 in 16 bits
    assert_eq!(format!("{v:x}"), "fef8");
    assert_eq!(format!("{v:X}"), "FEF8");
    assert_eq!(format!("{v:#x}"), "0xfef8");
}

/// `{:x}` always agrees with `to_bits()` — the core bit-accuracy invariant.
#[test]
fn q_hex_matches_to_bits() {
    let v = Q::<U12, U4>::from_f64(-16.5);
    assert_eq!(format!("{v:x}"), format!("{:04x}", v.to_bits()));
}

/// `{:b}` prints the full-width raw bit pattern; `#` adds the `0b` prefix.
#[test]
fn q_binary_is_raw_bits() {
    let v = Q::<U12, U4>::from_f64(16.5);
    assert_eq!(format!("{v:b}"), "0000000100001000");
    assert_eq!(format!("{v:#b}"), "0b0000000100001000");
}

/// `{:o}` prints the raw bits in octal, padded to the type width.
#[test]
fn q_octal_is_raw_bits() {
    let v = Q::<U12, U4>::from_f64(16.5); // 264 -> 0o410, 16 bits -> 6 digits
    assert_eq!(format!("{v:o}"), "000410");
}

/// `{:e}`/`{:E}` give scientific decimal of the value (delegated to `f64`).
#[test]
fn q_exp_is_scientific_value() {
    let v = Q::<U12, U4>::from_f64(16.5);
    assert_eq!(format!("{v:e}"), "1.65e1");
    assert_eq!(format!("{v:E}"), "1.65E1");
}

/// Debug is the exact self-describing form, with the value shown exactly.
#[test]
fn q_debug_exact_form() {
    let v = Q::<U12, U4>::from_f64(16.5);
    assert_eq!(format!("{v:?}"), "Q12.4(0x0108 = 16.5)");
}

/// UQ hex/binary print the raw bits, zero-padded to the type width.
#[test]
fn uq_hex_and_binary_are_raw_bits() {
    let v = UQ::<U1, U7>::from_f64(0.5); // 0x40 in an 8-bit type
    assert_eq!(format!("{v:x}"), "40");
    assert_eq!(format!("{v:#x}"), "0x40");
    assert_eq!(format!("{v:b}"), "01000000");
}

/// UQ Display honors precision and defaults to the shortest decimal.
#[test]
fn uq_display_precision_and_default() {
    let v = UQ::<U1, U7>::from_f64(0.5);
    assert_eq!(format!("{v:.3}"), "0.500");
    assert_eq!(format!("{v}"), "0.5");
}

/// UQ Debug is the exact self-describing form.
#[test]
fn uq_debug_exact_form() {
    let v = UQ::<U1, U7>::from_f64(0.5);
    assert_eq!(format!("{v:?}"), "UQ1.7(0x40 = 0.5)");
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
// min, max, clamp
// ---------------------------------------------------------------------------

/// Verifies min, max, and clamp behavior.
#[test]
fn q_min_max_clamp() {
    let a = Q::<U8, U8>::from_int(3);
    let b = Q::<U8, U8>::from_int(7);
    let lo = Q::<U8, U8>::from_int(4);
    let hi = Q::<U8, U8>::from_int(6);

    assert_eq!(a.min(b).to_int(), 3);
    assert_eq!(a.max(b).to_int(), 7);
    assert_eq!(a.clamp(lo, hi).to_int(), 4); // 3 clamped up to 4
    assert_eq!(b.clamp(lo, hi).to_int(), 6); // 7 clamped down to 6
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

// ---------------------------------------------------------------------------
// abs
// ---------------------------------------------------------------------------

/// Verifies abs returns magnitude for both negative and positive values.
#[test]
fn q_abs() {
    let a = Q::<U8, U8>::from_int(-5);
    assert_eq!(a.abs().to_int(), 5);
    let b = Q::<U8, U8>::from_int(3);
    assert_eq!(b.abs().to_int(), 3);
}

// ---------------------------------------------------------------------------
// Q: Saturating arithmetic
// ---------------------------------------------------------------------------

/// saturating_add clamps to MAX on positive overflow.
#[test]
fn q_saturating_add_overflow() {
    let max = Q::<U4, U4>::MAX;
    let one = Q::<U4, U4>::from_int(1);
    assert_eq!(max.saturating_add(one), Q::<U4, U4>::MAX);
}

/// saturating_add works normally when no overflow.
#[test]
fn q_saturating_add_normal() {
    let a = Q::<U8, U8>::from_int(3);
    let b = Q::<U8, U8>::from_int(4);
    assert_eq!(a.saturating_add(b).to_int(), 7);
}

/// saturating_sub clamps to MIN on negative overflow.
#[test]
fn q_saturating_sub_underflow() {
    let min = Q::<U4, U4>::MIN;
    let one = Q::<U4, U4>::from_int(1);
    assert_eq!(min.saturating_sub(one), Q::<U4, U4>::MIN);
}

/// saturating_sub works normally when no underflow.
#[test]
fn q_saturating_sub_normal() {
    let a = Q::<U8, U8>::from_int(7);
    let b = Q::<U8, U8>::from_int(3);
    assert_eq!(a.saturating_sub(b).to_int(), 4);
}

/// saturating_mul clamps to MAX on positive overflow.
#[test]
fn q_saturating_mul_overflow() {
    let max = Q::<U4, U4>::MAX;
    let two = Q::<U4, U4>::from_int(2);
    assert_eq!(max.saturating_mul(two), Q::<U4, U4>::MAX);
}

/// saturating_mul clamps to MIN on negative overflow.
#[test]
fn q_saturating_mul_negative_overflow() {
    let max = Q::<U4, U4>::MAX;
    let neg_two = Q::<U4, U4>::from_int(-2);
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
    let a = Q::<U8, U8>::from_int(5);
    assert_eq!(a.saturating_neg().to_int(), -5);
    let b = Q::<U8, U8>::from_int(-3);
    assert_eq!(b.saturating_neg().to_int(), 3);
}

// ---------------------------------------------------------------------------
// UQ: Saturating arithmetic
// ---------------------------------------------------------------------------

/// saturating_add clamps to MAX on overflow.
#[test]
fn uq_saturating_add_overflow() {
    let max = UQ::<U4, U4>::MAX;
    let one = UQ::<U4, U4>::from_int(1);
    assert_eq!(max.saturating_add(one), UQ::<U4, U4>::MAX);
}

/// saturating_add works normally when no overflow.
#[test]
fn uq_saturating_add_normal() {
    let a = UQ::<U8, U8>::from_int(3);
    let b = UQ::<U8, U8>::from_int(4);
    assert_eq!(a.saturating_add(b).to_int(), 7);
}

/// saturating_sub clamps to zero on underflow.
#[test]
fn uq_saturating_sub_underflow() {
    let a = UQ::<U4, U4>::from_int(1);
    let b = UQ::<U4, U4>::from_int(3);
    assert_eq!(a.saturating_sub(b), UQ::<U4, U4>::ZERO);
}

/// saturating_sub works normally when no underflow.
#[test]
fn uq_saturating_sub_normal() {
    let a = UQ::<U8, U8>::from_int(7);
    let b = UQ::<U8, U8>::from_int(3);
    assert_eq!(a.saturating_sub(b).to_int(), 4);
}

/// saturating_mul clamps to MAX on overflow.
#[test]
fn uq_saturating_mul_overflow() {
    let max = UQ::<U4, U4>::MAX;
    let two = UQ::<U4, U4>::from_int(2);
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
// Q4.12 pipeline-realistic tests
// ---------------------------------------------------------------------------

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
