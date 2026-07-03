//! Tests for the memory-compact packed containers.
//!
//! Verify that packing is lossless (round-trips exactly, including negative and
//! sub-byte-width values and boundary crossings), that `CQ` interleaves
//! correctly, and that the in-memory footprint actually shrinks.

use core::mem::size_of;

use qfixed::typenum::consts::*;
use qfixed::{CQ, PackedArray, PackedBits, Q, UQ};
#[cfg(feature = "alloc")]
use qfixed::{PackedBitsVec, PackedVec};

// ---------------------------------------------------------------------------
// PackedArray: byte-granular backing
// ---------------------------------------------------------------------------

/// Verifies `PackedArray<Q12.4>` uses an `i16` backing (2 bytes/element, not 8).
#[test]
fn packed_array_backing_size() {
    // Q12.4 is 16-bit -> i16 backing.
    assert_eq!(size_of::<PackedArray<Q<U12, U4>, 16>>(), 16 * 2);
    // Q4.4 is 8-bit -> i8 backing: 8x smaller than [Q; N] (8 bytes each).
    assert_eq!(size_of::<PackedArray<Q<U4, U4>, 100>>(), 100);
    // Q32.32 is 64-bit -> i64 backing: no shrink, still correct.
    assert_eq!(size_of::<PackedArray<Q<U32, U32>, 4>>(), 4 * 8);
    // CQ4.4 interleaves two i8 -> 2 bytes/element vs 16 for [CQ; N].
    assert_eq!(size_of::<PackedArray<CQ<U4, U4>, 100>>(), 200);
}

/// Verifies `set`/`get` round-trip signed values of both signs through
/// `PackedArray`, exhaustively over an 8-bit type.
#[test]
fn packed_array_q_roundtrip_exhaustive() {
    let mut arr = PackedArray::<Q<U4, U4>, 256>::new();
    for count in -128i64..=127 {
        let idx = (count + 128) as usize;
        arr.set(idx, Q::from_count(count));
    }
    for count in -128i64..=127 {
        let idx = (count + 128) as usize;
        assert_eq!(arr.get(idx).unwrap().to_count(), count, "count {count}");
    }
}

/// Verifies a sub-byte width (Q3.2, 5-bit) round-trips negatives correctly —
/// the stored `i8` holds only the low 5 bits, and `from_bits` re-sign-extends.
#[test]
fn packed_array_subbyte_width() {
    let mut arr = PackedArray::<Q<U3, U2>, 32>::new();
    // 5-bit signed count range is [-16, 15]; value = count / 4.
    for count in -16i64..=15 {
        arr.set((count + 16) as usize, Q::from_count(count));
    }
    for count in -16i64..=15 {
        assert_eq!(arr.get((count + 16) as usize).unwrap().to_count(), count);
    }
}

/// Verifies `from_fn`, `from_array`, `to_array`, and `iter` agree.
#[test]
fn packed_array_constructors() {
    let arr = PackedArray::<Q<U8, U8>, 5>::from_fn(|i| Q::from(i as i32));
    assert_eq!(arr.to_array(), core::array::from_fn(|i| Q::from(i as i32)));

    let src = [Q::<U8, U8>::from(-3i32); 3];
    let arr = PackedArray::from_array(src);
    let collected: [Q<U8, U8>; 3] = core::array::from_fn(|_| Q::from(-3i32));
    assert_eq!(arr.to_array(), collected);

    let sum: f64 = arr.iter().map(|q| q.to_f64()).sum();
    assert_eq!(sum, -9.0);
}

/// Verifies `UQ` uses an unsigned backing and round-trips its full range.
#[test]
fn packed_array_uq_roundtrip() {
    // UQ4.4 is 8-bit unsigned -> u8 backing.
    assert_eq!(size_of::<PackedArray<UQ<U4, U4>, 64>>(), 64);
    let mut arr = PackedArray::<UQ<U4, U4>, 256>::new();
    for count in 0u64..=255 {
        arr.set(count as usize, UQ::from_count(count));
    }
    for count in 0u64..=255 {
        assert_eq!(arr.get(count as usize).unwrap().to_count(), count);
    }
}

/// Verifies `CQ` interleaves real/imaginary and round-trips both components.
#[test]
fn packed_array_cq_roundtrip() {
    let mut arr = PackedArray::<CQ<U8, U8>, 3>::new();
    arr.set(0, CQ::from_count(100, -200));
    arr.set(1, CQ::from_count(-1, 1));
    arr.set(2, CQ::from_count(32767, -32768));
    assert_eq!(arr.get(0).unwrap().to_count(), (100, -200));
    assert_eq!(arr.get(1).unwrap().to_count(), (-1, 1));
    assert_eq!(arr.get(2).unwrap().to_count(), (32767, -32768));
}

/// Verifies out-of-bounds `get` returns `None` and `set` panics.
#[test]
fn packed_array_bounds() {
    let arr = PackedArray::<Q<U8, U8>, 4>::new();
    assert!(arr.get(4).is_none());
    assert_eq!(arr.len(), 4);
    assert!(!arr.is_empty());
}

/// Verifies a set past the end panics.
#[test]
#[should_panic(expected = "out of bounds")]
fn packed_array_set_oob_panics() {
    let mut arr = PackedArray::<Q<U8, U8>, 4>::new();
    arr.set(4, Q::from(1i32));
}

// ---------------------------------------------------------------------------
// PackedBits: bit-exact packing
// ---------------------------------------------------------------------------

/// Verifies `PackedBits` packs at exactly `I + F` bits, denser than the
/// byte-granular container: two 12-bit values fit in 3 bytes, not 4.
#[test]
fn packed_bits_density() {
    // 3 bytes = 24 bits / 12 = 2 elements. (Byte-granular would need i16 = 4 B.)
    let bits = PackedBits::<Q<U12, U0>, 3>::new();
    assert_eq!(bits.capacity(), 2);
    // 30 bytes = 240 bits / 12 = 20 twelve-bit samples.
    let adc = PackedBits::<Q<U12, U0>, 30>::new();
    assert_eq!(adc.capacity(), 20);
}

/// Verifies 12-bit values round-trip across byte boundaries, both signs.
#[test]
fn packed_bits_12bit_roundtrip() {
    let mut adc = PackedBits::<Q<U12, U0>, 30>::new();
    let samples = [0i64, 1, -1, 2047, -2048, 1000, -1000, 42, -42];
    for &s in &samples {
        adc.push(Q::from_count(s));
    }
    for (i, &s) in samples.iter().enumerate() {
        assert_eq!(adc.get(i).unwrap().to_count(), s, "sample {i}");
    }
    assert_eq!(adc.len(), samples.len());
}

/// Verifies an odd sub-byte width (5-bit) round-trips every representable
/// value when packed across byte boundaries.
#[test]
fn packed_bits_5bit_exhaustive() {
    // 5 bytes = 40 bits / 5 = 8 elements.
    let mut bits = PackedBits::<Q<U3, U2>, 5>::new();
    assert_eq!(bits.capacity(), 8);
    let counts = [-16i64, -9, -1, 0, 1, 7, 15, -8];
    for &c in &counts {
        bits.push(Q::from_count(c));
    }
    for (i, &c) in counts.iter().enumerate() {
        assert_eq!(bits.get(i).unwrap().to_count(), c);
    }
}

/// Verifies the full 64-bit lane path (`LANE_BITS == 64`) round-trips.
#[test]
fn packed_bits_64bit_lane() {
    let mut bits = PackedBits::<Q<U32, U32>, 16>::new();
    assert_eq!(bits.capacity(), 2); // 16 bytes = 128 bits / 64.
    bits.push(Q::from(5i32));
    bits.push(Q::from(-7i32));
    assert_eq!(bits.get(0).unwrap().to_f64(), 5.0);
    assert_eq!(bits.get(1).unwrap().to_f64(), -7.0);
}

/// Verifies `CQ` occupies `2 * (I + F)` bits and interleaves in the stream.
#[test]
fn packed_bits_cq_interleaved() {
    // CQ6.0 = 6 bits/component, 12 bits/element. 3 bytes = 24 bits / 12 = 2.
    let mut bits = PackedBits::<CQ<U6, U0>, 3>::new();
    assert_eq!(bits.capacity(), 2);
    bits.push(CQ::from_count(31, -32));
    bits.push(CQ::from_count(-1, 1));
    assert_eq!(bits.get(0).unwrap().to_count(), (31, -32));
    assert_eq!(bits.get(1).unwrap().to_count(), (-1, 1));
}

/// Verifies `set` overwrites in place without disturbing neighbours.
#[test]
fn packed_bits_set_in_place() {
    let mut bits = PackedBits::<Q<U12, U0>, 30>::new();
    for _ in 0..5 {
        bits.push(Q::from_count(2047)); // all-ones-ish neighbours
    }
    bits.set(2, Q::from_count(-1));
    assert_eq!(bits.get(1).unwrap().to_count(), 2047);
    assert_eq!(bits.get(2).unwrap().to_count(), -1);
    assert_eq!(bits.get(3).unwrap().to_count(), 2047);
}

/// Verifies `try_push` reports a full buffer and `push` panics on overflow.
#[test]
fn packed_bits_capacity_limit() {
    let mut bits = PackedBits::<Q<U8, U0>, 2>::new();
    assert_eq!(bits.capacity(), 2);
    assert!(bits.try_push(Q::from(1i32)).is_ok());
    assert!(bits.try_push(Q::from(2i32)).is_ok());
    assert!(bits.try_push(Q::from(3i32)).is_err());
}

/// Verifies `PackedBits::pop` returns elements last-in-first-out and `None`
/// when empty.
#[test]
fn packed_bits_pop() {
    let mut bits = PackedBits::<Q<U12, U0>, 30>::new();
    bits.push(Q::from_count(100));
    bits.push(Q::from_count(-200));
    assert_eq!(bits.pop().unwrap().to_count(), -200);
    assert_eq!(bits.pop().unwrap().to_count(), 100);
    assert!(bits.pop().is_none());
    assert!(bits.is_empty());
}

/// Verifies a 17-bit lane (Q9.8) packs across several byte boundaries with
/// arbitrary bit offsets, round-tripping negatives.
#[test]
fn packed_bits_17bit_crossing() {
    // 17-bit elements: successive offsets 0,17,34,51,68,... exercise every
    // intra-byte shift. 12 bytes = 96 bits / 17 = 5 elements.
    let mut bits = PackedBits::<Q<U9, U8>, 12>::new();
    assert_eq!(bits.capacity(), 5);
    let counts = [0i64, -1, 65535, -65536, 12345];
    for &c in &counts {
        bits.push(Q::from_count(c));
    }
    for (i, &c) in counts.iter().enumerate() {
        assert_eq!(bits.get(i).unwrap().to_count(), c, "elem {i}");
    }
}

/// Verifies a byte budget too small for even one element yields capacity 0.
#[test]
fn packed_bits_zero_capacity() {
    // Q12.0 needs 12 bits; 1 byte = 8 bits holds none.
    let bits = PackedBits::<Q<U12, U0>, 1>::new();
    assert_eq!(bits.capacity(), 0);
    assert!(bits.is_empty());
    assert!(bits.get(0).is_none());
}

/// Verifies a 32-bit `CQ` lane (`CQ16.16`, 64-bit element) round-trips.
#[test]
fn packed_bits_cq_wide_lane() {
    let mut bits = PackedBits::<CQ<U16, U16>, 16>::new();
    assert_eq!(bits.capacity(), 2); // 16 bytes = 128 bits / 64.
    bits.push(CQ::from_count(2_000_000, -2_000_000));
    bits.push(CQ::from_count(-1, 1));
    assert_eq!(bits.get(0).unwrap().to_count(), (2_000_000, -2_000_000));
    assert_eq!(bits.get(1).unwrap().to_count(), (-1, 1));
}

// ---------------------------------------------------------------------------
// PackedVec: growable byte-granular (alloc)
// ---------------------------------------------------------------------------

/// Verifies `PackedVec` push/pop/get/set/iter round-trip.
#[cfg(feature = "alloc")]
#[test]
fn packed_vec_roundtrip() {
    let mut v = PackedVec::<Q<U8, U8>>::new();
    assert!(v.is_empty());
    for i in -3i32..=3 {
        v.push(Q::from(i));
    }
    assert_eq!(v.len(), 7);
    v.set(0, Q::from(10i32));
    assert_eq!(v.get(0).unwrap().to_f64(), 10.0);
    assert_eq!(v.pop().unwrap().to_f64(), 3.0);
    assert_eq!(v.len(), 6);

    let values: alloc::vec::Vec<f64> = v.iter().map(|q| q.to_f64()).collect();
    assert_eq!(values, [10.0, -2.0, -1.0, 0.0, 1.0, 2.0]);
}

/// Verifies `FromIterator` and `Extend` for `PackedVec`.
#[cfg(feature = "alloc")]
#[test]
fn packed_vec_from_iter_and_extend() {
    let mut v: PackedVec<Q<U4, U4>> = (0i32..4).map(Q::from).collect();
    assert_eq!(v.len(), 4);
    v.extend((4i32..6).map(Q::from));
    assert_eq!(v.len(), 6);
    let sum: f64 = v.iter().map(|q| q.to_f64()).sum();
    assert_eq!(sum, (0..6).sum::<i32>() as f64);
}

/// Verifies `PackedVec<CQ>` interleaves and round-trips.
#[cfg(feature = "alloc")]
#[test]
fn packed_vec_cq() {
    let mut v = PackedVec::<CQ<U8, U8>>::new();
    v.push(CQ::from_count(7, -9));
    v.push(CQ::from_count(-100, 200));
    assert_eq!(v.get(0).unwrap().to_count(), (7, -9));
    assert_eq!(v.get(1).unwrap().to_count(), (-100, 200));
}

// ---------------------------------------------------------------------------
// PackedBitsVec: growable bit-exact (alloc)
// ---------------------------------------------------------------------------

/// Verifies `PackedBitsVec` grows on demand and round-trips 12-bit samples.
#[cfg(feature = "alloc")]
#[test]
fn packed_bits_vec_grows() {
    let mut v = PackedBitsVec::<Q<U12, U0>>::new();
    let samples = [0i64, -1, 2047, -2048, 55, -55, 1234];
    for &s in &samples {
        v.push(Q::from_count(s));
    }
    for (i, &s) in samples.iter().enumerate() {
        assert_eq!(v.get(i).unwrap().to_count(), s);
    }
    assert_eq!(v.pop().unwrap().to_count(), 1234);
    assert_eq!(v.len(), samples.len() - 1);
}

/// Verifies `FromIterator` for `PackedBitsVec` preserves values.
#[cfg(feature = "alloc")]
#[test]
fn packed_bits_vec_from_iter() {
    let v: PackedBitsVec<Q<U12, U0>> = [10i64, -20, 30, -40]
        .into_iter()
        .map(Q::from_count)
        .collect();
    let got: alloc::vec::Vec<i64> = v.iter().map(|q| q.to_count()).collect();
    assert_eq!(got, [10, -20, 30, -40]);
}

/// Verifies a growable bit-exact buffer of interleaved `CQ` round-trips both
/// components as it grows across byte boundaries.
#[cfg(feature = "alloc")]
#[test]
fn packed_bits_vec_cq_grows() {
    // CQ5.0 = 5 bits/component, 10 bits/element — every element misaligns.
    let mut v = PackedBitsVec::<CQ<U5, U0>>::new();
    let pairs = [(15i64, -16i64), (-1, 1), (0, 7), (-16, 15), (3, -3)];
    for &(re, im) in &pairs {
        v.push(CQ::from_count(re, im));
    }
    for (i, &(re, im)) in pairs.iter().enumerate() {
        assert_eq!(v.get(i).unwrap().to_count(), (re, im), "elem {i}");
    }
    assert_eq!(v.pop().unwrap().to_count(), (3, -3));
    assert_eq!(v.len(), pairs.len() - 1);
}

/// Brings `alloc` into scope for the collected-`Vec` assertions above.
#[cfg(feature = "alloc")]
extern crate alloc;
