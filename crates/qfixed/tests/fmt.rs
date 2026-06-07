//! Tests for `Q`/`UQ` `Display`, `Debug`, and the hex/binary/octal/scientific
//! formatting traits.

use qfixed::typenum::consts::*;
use qfixed::{Q, UQ};

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
