//! Serde round-trip tests for `Q`, `UQ`, and `CQ` (requires the `serde`
//! feature).
//!
//! Uses `serde_test::assert_tokens`, which checks both serialization and
//! deserialization against the expected token stream — so it exercises the
//! `from_bits` reconstruction path, including for negative values whose masked
//! words serialize as large positives.

#![cfg(feature = "serde")]

use qfixed::typenum::consts::*;
use qfixed::{CQ, Q, UQ};
use serde_test::{Token, assert_tokens};

/// Round-trips signed `Q` values of both signs through serde (`u64` bits).
#[test]
fn q_serde_roundtrip() {
    // Q8.8 is 16-bit: count range [-32768, 32767].
    for &count in &[-32768i64, -1000, -16, -1, 0, 1, 16, 1000, 32767] {
        let x = Q::<U8, U8>::from_count(count);
        assert_tokens(&x, &[Token::U64(x.to_bits())]);
    }
}

/// Round-trips unsigned `UQ` values (including high-bit-set) through serde.
#[test]
fn uq_serde_roundtrip() {
    // UQ4.4 is 8-bit: count range [0, 255].
    for &count in &[0u64, 1, 0x80, 0xC8, 0xFF] {
        let x = UQ::<U4, U4>::from_count(count);
        assert_tokens(&x, &[Token::U64(x.to_bits())]);
    }
}

/// Round-trips complex `CQ` values (including negative components) through
/// serde, which serializes as a `(u64, u64)` tuple of raw bits.
#[test]
fn cq_serde_roundtrip() {
    let values = [
        CQ::<U8, U8>::from_f64(1.5, -2.25),
        CQ::<U8, U8>::from_f64(-3.0, -4.0),
        CQ::<U8, U8>::from_f64(0.0, 0.0),
    ];
    for &x in &values {
        let (re, im) = x.to_bits();
        assert_tokens(
            &x,
            &[
                Token::Tuple { len: 2 },
                Token::U64(re),
                Token::U64(im),
                Token::TupleEnd,
            ],
        );
    }
}
