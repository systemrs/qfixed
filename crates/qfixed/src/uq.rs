//! Unsigned fixed-point type `UQ<I, F>`.

use core::marker::PhantomData;

use typenum::Unsigned;

use crate::error::FixedError;

/// Unsigned fixed-point number with `I` integer bits and `F` fractional bits.
///
/// `I` and `F` are [`typenum`](crate::typenum) unsigned integers, e.g.
/// `UQ<U4, U14>`.
///
/// Total bit width = `I + F`, must satisfy `1 <= I + F <= 64`.
/// Internally backed by `u64`, masked to `I + F` bits.
///
/// # Notation
///
/// Follows TI-style UQ notation: `UQ<U1, U7>` is an 8-bit unsigned value with
/// 1 integer bit and 7 fractional bits, representing values in increments of
/// 1/128.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UQ<I, F>(pub(crate) u64, PhantomData<(I, F)>);

impl<I: Unsigned, F: Unsigned> UQ<I, F> {
    /// Total number of integer bits in this fixed-point type.
    pub const INTEGER_BITS: u32 = I::U32;

    /// Total number of fractional bits in this fixed-point type.
    pub const FRACTIONAL_BITS: u32 = F::U32;

    /// Total number of bits in this fixed-point type.
    pub const TOTAL_BITS: u32 = Self::INTEGER_BITS + Self::FRACTIONAL_BITS;

    /// Bitmask covering the valid bits.
    const MASK: u64 = if Self::TOTAL_BITS >= 64 {
        u64::MAX
    } else {
        (1u64 << Self::TOTAL_BITS) - 1
    };

    /// The value zero (0.0).
    pub const ZERO: Self = Self(0, PhantomData);

    /// The value one (1.0).
    ///
    /// Only available when `I >= 1` (at least one integer bit).
    ///
    /// # Panics
    ///
    /// Panics at compile time if `I == 0`.
    pub const ONE: Self = {
        assert!(Self::TOTAL_BITS > 0, "UQ type must have at least 1 bit");
        assert!(Self::TOTAL_BITS <= 64, "UQ type cannot exceed 64 bits");
        assert!(
            Self::INTEGER_BITS >= 1,
            "UQ<I,F> with I==0 cannot represent 1.0"
        );
        Self(1u64 << Self::FRACTIONAL_BITS, PhantomData)
    };

    /// Maximum representable value.
    ///
    /// For `UQ<I, F>`, this is `2^I - 2^(-F)`.
    pub const MAX: Self = {
        assert!(Self::TOTAL_BITS > 0, "UQ type must have at least 1 bit");
        assert!(Self::TOTAL_BITS <= 64, "UQ type cannot exceed 64 bits");
        Self(Self::MASK, PhantomData)
    };

    /// Minimum representable value (always zero for unsigned).
    pub const MIN: Self = Self(0, PhantomData);

    /// Forces compile-time validation of the type parameters.
    ///
    /// # Panics
    ///
    /// Panics at compile time if `I + F` is 0 or exceeds 64.
    #[inline(always)]
    pub(crate) const fn check() {
        assert!(Self::TOTAL_BITS > 0, "UQ type must have at least 1 bit");
        assert!(Self::TOTAL_BITS <= 64, "UQ type cannot exceed 64 bits");
    }

    /// Constructs from a raw bit pattern (the low `I + F` bits).
    ///
    /// Any bits above the low `I + F` are **masked off** (never panics),
    /// matching RTL register truncation. For a checked conversion use
    /// [`try_from_bits`](Self::try_from_bits).
    ///
    /// # Arguments
    ///
    /// * `bits` - The raw bit pattern; only the low `I + F` bits are used.
    ///
    /// # Returns
    ///
    /// A new `UQ<I, F>` holding the masked value.
    #[inline]
    pub const fn from_bits(bits: u64) -> Self {
        Self::from_raw(bits)
    }

    /// Constructs from a raw bit pattern, erroring if it does not fit.
    ///
    /// Like [`from_bits`](Self::from_bits) but returns
    /// [`FixedError::OutOfRange`] instead of masking when `bits` sets any bit
    /// above the low `I + F`.
    ///
    /// # Arguments
    ///
    /// * `bits` - The raw bit pattern.
    ///
    /// # Returns
    ///
    /// The reconstructed `UQ<I, F>`, or [`FixedError::OutOfRange`].
    ///
    /// # Errors
    ///
    /// Returns [`FixedError::OutOfRange`] if `bits != bits & MASK`.
    #[inline]
    pub const fn try_from_bits(bits: u64) -> Result<Self, FixedError> {
        if bits == bits & Self::MASK {
            Ok(Self::from_raw(bits))
        } else {
            Err(FixedError::OutOfRange)
        }
    }

    /// Extracts the raw bit representation.
    ///
    /// # Returns
    ///
    /// The raw fixed-point value as a `u64`.
    #[inline]
    pub const fn to_bits(self) -> u64 {
        self.0
    }

    /// Returns the internal raw value.
    ///
    /// # Returns
    ///
    /// The stored u64 (already masked to `I + F` bits).
    #[inline]
    pub(crate) const fn raw(self) -> u64 {
        self.0
    }

    /// Constructs from a masked u64 without the `from_bits` debug range
    /// check.
    ///
    /// # Arguments
    ///
    /// * `raw` - The value to store, which will be masked to `I + F` bits.
    ///
    /// # Returns
    ///
    /// A new `UQ<I, F>` with the masked value.
    #[inline]
    pub(crate) const fn from_raw(raw: u64) -> Self {
        Self::check();
        Self(raw & Self::MASK, PhantomData)
    }

    /// Constructs from a count of fractional LSB units.
    ///
    /// One count is `2^-F` (one LSB), so the resulting value is
    /// `count * 2^-F` and `count` is the raw register word. An out-of-range
    /// `count` is **wrapped** (masked to `I + F` bits, like an RTL register) —
    /// it never panics. For a checked conversion use
    /// [`try_from_count`](Self::try_from_count); to build a whole number use
    /// the [`From`] conversions (e.g. `UQ::from(3u32)`).
    ///
    /// # Arguments
    ///
    /// * `count` - The number of `2^-F` units.
    ///
    /// # Returns
    ///
    /// A new `UQ<I, F>` representing `count * 2^-F`, wrapped to range.
    #[inline]
    pub const fn from_count(count: u64) -> Self {
        Self::from_raw(count)
    }

    /// Constructs from a count of LSB units, erroring if out of range.
    ///
    /// Like [`from_count`](Self::from_count) but returns
    /// [`FixedError::OutOfRange`] instead of wrapping when `count` exceeds
    /// `MAX`.
    ///
    /// # Arguments
    ///
    /// * `count` - The number of `2^-F` units.
    ///
    /// # Returns
    ///
    /// The `UQ<I, F>` value, or [`FixedError::OutOfRange`].
    ///
    /// # Errors
    ///
    /// Returns [`FixedError::OutOfRange`] if `count` exceeds `MAX`.
    #[inline]
    pub const fn try_from_count(count: u64) -> Result<Self, FixedError> {
        if count <= Self::MAX.0 {
            Ok(Self::from_raw(count))
        } else {
            Err(FixedError::OutOfRange)
        }
    }

    /// Returns the value as a count of fractional LSB units.
    ///
    /// One count is `2^-F` (one LSB), so this returns `value * 2^F` — the raw
    /// register word. Lossless; the exact inverse of
    /// [`from_count`](Self::from_count). Equal to [`to_bits`](Self::to_bits)
    /// for unsigned values.
    ///
    /// # Returns
    ///
    /// The count of `2^-F` units.
    #[inline]
    pub const fn to_count(self) -> u64 {
        self.0
    }

    /// Constructs from `f64` by quantizing to the nearest representable
    /// value.
    ///
    /// # Arguments
    ///
    /// * `val` - The floating-point value to quantize.
    ///
    /// # Returns
    ///
    /// A new `UQ<I, F>` representing the quantized value. Out-of-range or
    /// non-finite inputs produce a wrapped/saturated value rather than
    /// panicking; use [`try_from_f64`](Self::try_from_f64) for a checked
    /// conversion.
    #[inline]
    pub fn from_f64(val: f64) -> Self {
        Self::check();
        let scale = (1u64 << Self::FRACTIONAL_BITS) as f64;
        let raw = (val * scale) as u64;
        Self::from_raw(raw)
    }

    /// Constructs from `f64`, erroring on non-finite or out-of-range input.
    ///
    /// # Arguments
    ///
    /// * `val` - The floating-point value to quantize.
    ///
    /// # Returns
    ///
    /// The quantized `UQ<I, F>` value.
    ///
    /// # Errors
    ///
    /// Returns [`FixedError::NotFinite`] if `val` is NaN or infinite, or
    /// [`FixedError::OutOfRange`] if it is negative or exceeds `MAX`.
    #[inline]
    pub fn try_from_f64(val: f64) -> Result<Self, FixedError> {
        Self::check();
        if !val.is_finite() {
            return Err(FixedError::NotFinite);
        }
        let scale = (1u64 << Self::FRACTIONAL_BITS) as f64;
        let scaled = val * scale;
        if scaled < 0.0 || scaled > Self::MAX.0 as f64 {
            return Err(FixedError::OutOfRange);
        }
        Ok(Self::from_raw(scaled as u64))
    }

    /// Converts to `f64`.
    ///
    /// # Returns
    ///
    /// The fixed-point value as a floating-point approximation.
    #[inline]
    pub fn to_f64(self) -> f64 {
        let scale = (1u64 << Self::FRACTIONAL_BITS) as f64;
        self.0 as f64 / scale
    }

    /// Returns the minimum of `self` and `other`.
    ///
    /// # Arguments
    ///
    /// * `other` - The value to compare against.
    ///
    /// # Returns
    ///
    /// The smaller of the two values.
    #[inline]
    pub const fn min(self, other: Self) -> Self {
        if self.0 < other.0 { self } else { other }
    }

    /// Returns the maximum of `self` and `other`.
    ///
    /// # Arguments
    ///
    /// * `other` - The value to compare against.
    ///
    /// # Returns
    ///
    /// The larger of the two values.
    #[inline]
    pub const fn max(self, other: Self) -> Self {
        if self.0 > other.0 { self } else { other }
    }

    /// Clamps `self` to the range `[lo, hi]`.
    ///
    /// # Arguments
    ///
    /// * `lo` - The lower bound.
    /// * `hi` - The upper bound.
    ///
    /// # Returns
    ///
    /// `self` clamped to `[lo, hi]`.
    #[inline]
    pub const fn clamp(self, lo: Self, hi: Self) -> Self {
        self.max(lo).min(hi)
    }

    /// Wrapping addition (always wraps, never traps).
    ///
    /// # Arguments
    ///
    /// * `rhs` - The value to add.
    ///
    /// # Returns
    ///
    /// The sum, wrapped to `I + F` bits.
    #[inline]
    pub const fn wrapping_add(self, rhs: Self) -> Self {
        Self::from_raw(self.0.wrapping_add(rhs.0))
    }

    /// Wrapping subtraction (always wraps, never traps).
    ///
    /// # Arguments
    ///
    /// * `rhs` - The value to subtract.
    ///
    /// # Returns
    ///
    /// The difference, wrapped to `I + F` bits.
    #[inline]
    pub const fn wrapping_sub(self, rhs: Self) -> Self {
        Self::from_raw(self.0.wrapping_sub(rhs.0))
    }

    /// Wrapping same-type multiply: `(self * rhs) >> F`, truncated.
    ///
    /// The full product is computed in 128 bits, then the fractional
    /// point is realigned by shifting right by `F`.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The value to multiply by.
    ///
    /// # Returns
    ///
    /// The truncated product in the same `UQ<I, F>` format.
    #[inline]
    pub const fn wrapping_mul(self, rhs: Self) -> Self {
        let product = self.0 as u128 * rhs.0 as u128;
        let shifted = (product >> Self::FRACTIONAL_BITS) as u64;
        Self::from_raw(shifted)
    }

    /// Wrapping same-type divide: `(self << F) / rhs`, truncated.
    ///
    /// The numerator is widened in 128 bits before division to preserve
    /// fractional precision.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The divisor.
    ///
    /// # Returns
    ///
    /// The truncated quotient in the same `UQ<I, F>` format.
    ///
    /// # Panics
    ///
    /// Panics if `rhs` is zero (in all build profiles). Use
    /// [`checked_div`](Self::checked_div) to handle a zero divisor.
    #[inline]
    pub const fn wrapping_div(self, rhs: Self) -> Self {
        let numer = (self.0 as u128) << Self::FRACTIONAL_BITS;
        let result = (numer / rhs.0 as u128) as u64;
        Self::from_raw(result)
    }

    /// Checked same-type divide: `(self << F) / rhs`, or `None` if `rhs` is
    /// zero.
    ///
    /// The numerator is widened in 128 bits before division to preserve
    /// fractional precision; the quotient is truncated to `I + F` bits.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The divisor.
    ///
    /// # Returns
    ///
    /// `Some(quotient)` in the same `UQ<I, F>` format, or `None` if `rhs` is
    /// zero.
    #[inline]
    pub const fn checked_div(self, rhs: Self) -> Option<Self> {
        if rhs.0 == 0 {
            None
        } else {
            let numer = (self.0 as u128) << Self::FRACTIONAL_BITS;
            let result = (numer / rhs.0 as u128) as u64;
            Some(Self::from_raw(result))
        }
    }

    /// Saturating addition: clamps to `MAX` on overflow.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The value to add.
    ///
    /// # Returns
    ///
    /// The sum, clamped to the representable range.
    #[inline]
    pub const fn saturating_add(self, rhs: Self) -> Self {
        if Self::TOTAL_BITS >= 64 {
            Self(self.0.saturating_add(rhs.0), PhantomData)
        } else {
            let sum = self.0 as u128 + rhs.0 as u128;
            if sum > Self::MASK as u128 {
                Self::MAX
            } else {
                Self::from_raw(sum as u64)
            }
        }
    }

    /// Saturating subtraction: clamps to zero on underflow.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The value to subtract.
    ///
    /// # Returns
    ///
    /// The difference, or zero if `rhs > self`.
    #[inline]
    pub const fn saturating_sub(self, rhs: Self) -> Self {
        if rhs.0 > self.0 {
            Self::ZERO
        } else {
            Self::from_raw(self.0 - rhs.0)
        }
    }

    /// Saturating same-type multiply: `(self * rhs) >> F`, clamped to
    /// `MAX`.
    ///
    /// The full product is computed in 128 bits, then the fractional
    /// point is realigned by shifting right by `F`.
    /// The result is clamped rather than truncated.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The value to multiply by.
    ///
    /// # Returns
    ///
    /// The product clamped to the representable range.
    #[inline]
    pub const fn saturating_mul(self, rhs: Self) -> Self {
        let product = self.0 as u128 * rhs.0 as u128;
        let shifted = product >> Self::FRACTIONAL_BITS;
        if shifted > Self::MASK as u128 {
            Self::MAX
        } else {
            Self::from_raw(shifted as u64)
        }
    }

    /// Widening addition with full-precision output.
    ///
    /// The output type `UQ<IO, FO>` must be wide enough to hold the sum
    /// without overflow (at least one extra integer bit).
    ///
    /// # Arguments
    ///
    /// * `rhs` - The value to add.
    ///
    /// # Returns
    ///
    /// The exact sum in `UQ<IO, FO>` format.
    ///
    /// # Compile-time checks
    ///
    /// Fails to compile if the output type is too narrow (`IO + FO <= I + F`).
    #[inline]
    pub fn widening_add<IO: Unsigned, FO: Unsigned>(self, rhs: Self) -> UQ<IO, FO> {
        const {
            assert!(
                UQ::<IO, FO>::TOTAL_BITS > Self::TOTAL_BITS,
                "widening_add: output type too narrow (needs at least one more integer bit than the input)"
            );
        }
        let shift = FO::U32 as i32 - F::U32 as i32;
        let a = if shift >= 0 {
            (self.0 as u128) << shift
        } else {
            (self.0 as u128) >> (-shift)
        };
        let b = if shift >= 0 {
            (rhs.0 as u128) << shift
        } else {
            (rhs.0 as u128) >> (-shift)
        };
        UQ::<IO, FO>::from_raw((a + b) as u64)
    }

    /// Widening subtraction with signed output.
    ///
    /// Returns a signed `Q<IO, FO>` because `UQ - UQ` can be negative.
    /// The output type must be wide enough to hold the difference.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The value to subtract.
    ///
    /// # Returns
    ///
    /// The exact difference as a signed `Q<IO, FO>`.
    ///
    /// # Compile-time checks
    ///
    /// Fails to compile if the output type is too narrow (`IO + FO <= I + F`).
    #[inline]
    pub fn widening_sub<IO: Unsigned, FO: Unsigned>(self, rhs: Self) -> crate::q::Q<IO, FO> {
        const {
            assert!(
                crate::q::Q::<IO, FO>::TOTAL_BITS > Self::TOTAL_BITS,
                "widening_sub: output type too narrow (needs at least one more integer bit than the input)"
            );
        }
        let shift = FO::U32 as i32 - F::U32 as i32;
        let a = if shift >= 0 {
            (self.0 as i128) << shift
        } else {
            (self.0 as i128) >> (-shift)
        };
        let b = if shift >= 0 {
            (rhs.0 as i128) << shift
        } else {
            (rhs.0 as i128) >> (-shift)
        };
        crate::q::Q::<IO, FO>::from_raw((a - b) as i64)
    }

    /// Widening multiply with full-precision output.
    ///
    /// The output type `UQ<IO, FO>` must be wide enough to hold the
    /// product without loss.
    /// The output type is typically inferred from the assignment target.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The right-hand operand.
    ///
    /// # Returns
    ///
    /// The full-precision product in `UQ<IO, FO>` format.
    ///
    /// # Compile-time checks
    ///
    /// Fails to compile if the output type is too narrow
    /// (`IO + FO < I + F + I2 + F2`).
    #[inline]
    pub fn widening_mul<I2: Unsigned, F2: Unsigned, IO: Unsigned, FO: Unsigned>(
        self,
        rhs: UQ<I2, F2>,
    ) -> UQ<IO, FO> {
        const {
            assert!(
                UQ::<IO, FO>::TOTAL_BITS >= Self::TOTAL_BITS + UQ::<I2, F2>::TOTAL_BITS,
                "widening_mul: output type too narrow to hold the full-width product"
            );
        }
        let product = self.0 as u128 * rhs.0 as u128;
        let frac_in = F::U32 + F2::U32;
        let shift = frac_in as i32 - FO::U32 as i32;
        let result = if shift > 0 {
            product >> shift
        } else {
            product << (-shift)
        };
        UQ::<IO, FO>::from_raw(result as u64)
    }
}
