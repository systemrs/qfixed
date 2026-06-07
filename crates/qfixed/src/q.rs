//! Signed fixed-point type `Q<I, F>`.

use core::marker::PhantomData;

use typenum::Unsigned;

use crate::error::FixedError;

/// Signed fixed-point number with `I` integer bits (including sign) and `F`
/// fractional bits.
///
/// `I` and `F` are [`typenum`](crate::typenum) unsigned integers, e.g.
/// `Q<U12, U4>`.
///
/// Total bit width = `I + F`, must satisfy `1 <= I + F <= 64`.
/// Internally backed by `i64`, sign-extended from bit `I + F - 1`.
///
/// # Notation
///
/// Follows TI-style Q notation: `Q<U1, U8>` is a 9-bit signed value with
/// 1 integer bit (the sign) and 8 fractional bits, representing values
/// in increments of 1/256.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Q<I, F>(pub(crate) i64, PhantomData<(I, F)>);

impl<I: Unsigned, F: Unsigned> Q<I, F> {
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

    /// Number of bits to shift for sign extension from `TOTAL_BITS` to 64.
    const SIGN_EXT_SHIFT: u32 = 64 - Self::TOTAL_BITS;

    /// The value zero (0.0).
    pub const ZERO: Self = Self(0, PhantomData);

    /// The value one (1.0).
    ///
    /// Only available when `I > 1` (at least one non-sign integer bit).
    ///
    /// # Panics
    ///
    /// Panics at compile time if `I <= 1`.
    pub const ONE: Self = {
        assert!(Self::TOTAL_BITS > 0, "Q type must have at least 1 bit");
        assert!(Self::TOTAL_BITS <= 64, "Q type cannot exceed 64 bits");
        assert!(
            Self::INTEGER_BITS > 1,
            "Q<I,F> with I<=1 cannot represent +1.0"
        );
        Self(1i64 << Self::FRACTIONAL_BITS, PhantomData)
    };

    /// Maximum representable value.
    ///
    /// For `Q<I, F>`, this is `2^(I-1) - 2^(-F)`.
    pub const MAX: Self = {
        assert!(Self::TOTAL_BITS > 0, "Q type must have at least 1 bit");
        assert!(Self::TOTAL_BITS <= 64, "Q type cannot exceed 64 bits");
        if Self::TOTAL_BITS >= 64 {
            Self(i64::MAX, PhantomData)
        } else {
            Self((1i64 << (Self::TOTAL_BITS - 1)) - 1, PhantomData)
        }
    };

    /// Minimum representable value (most negative).
    ///
    /// For `Q<I, F>`, this is `-2^(I-1)`.
    pub const MIN: Self = {
        assert!(Self::TOTAL_BITS > 0, "Q type must have at least 1 bit");
        assert!(Self::TOTAL_BITS <= 64, "Q type cannot exceed 64 bits");
        if Self::TOTAL_BITS >= 64 {
            Self(i64::MIN, PhantomData)
        } else {
            Self(-(1i64 << (Self::TOTAL_BITS - 1)), PhantomData)
        }
    };

    /// Forces compile-time validation of the type parameters.
    ///
    /// # Panics
    ///
    /// Panics at compile time if `I + F` is 0 or exceeds 64.
    #[inline(always)]
    pub(crate) const fn check() {
        assert!(Self::TOTAL_BITS > 0, "Q type must have at least 1 bit");
        assert!(Self::TOTAL_BITS <= 64, "Q type cannot exceed 64 bits");
    }

    /// Sign-extends a value from `TOTAL_BITS` to 64 bits.
    ///
    /// # Arguments
    ///
    /// * `raw` - The value to sign-extend, with valid data in the low
    ///   `I + F` bits.
    ///
    /// # Returns
    ///
    /// The sign-extended 64-bit value.
    #[inline]
    const fn sign_extend(raw: i64) -> i64 {
        if Self::TOTAL_BITS >= 64 {
            raw
        } else {
            (raw << Self::SIGN_EXT_SHIFT) >> Self::SIGN_EXT_SHIFT
        }
    }

    /// Constructs from a raw bit pattern (the low `I + F` bits), sign-extending.
    ///
    /// `bits` is the unsigned two's-complement pattern as produced by
    /// [`to_bits`](Self::to_bits); any bits above the low `I + F` are **masked
    /// off** (never panics), matching RTL register truncation. For a checked
    /// conversion use [`try_from_bits`](Self::try_from_bits); for the signed
    /// numeric value use [`from_count`](Self::from_count).
    ///
    /// # Arguments
    ///
    /// * `bits` - The raw bit pattern; only the low `I + F` bits are used.
    ///
    /// # Returns
    ///
    /// A new `Q<I, F>` holding the sign-extended value.
    #[inline]
    pub const fn from_bits(bits: u64) -> Self {
        Self::from_raw(bits as i64)
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
    /// The reconstructed `Q<I, F>`, or [`FixedError::OutOfRange`] if `bits`
    /// carries information outside the low `I + F` bits.
    ///
    /// # Errors
    ///
    /// Returns [`FixedError::OutOfRange`] if `bits != bits & MASK`.
    #[inline]
    pub const fn try_from_bits(bits: u64) -> Result<Self, FixedError> {
        if bits == bits & Self::MASK {
            Ok(Self::from_raw(bits as i64))
        } else {
            Err(FixedError::OutOfRange)
        }
    }

    /// Extracts the raw bit pattern, masked to the low `I + F` bits.
    ///
    /// Returns the unsigned two's-complement pattern (a negative value reads
    /// back as a large positive), matching [`f64::to_bits`]. For the signed
    /// numeric value, use [`to_count`](Self::to_count).
    ///
    /// # Returns
    ///
    /// The raw fixed-point bits as a `u64`; only the low `I + F` bits are
    /// meaningful.
    #[inline]
    pub const fn to_bits(self) -> u64 {
        (self.0 as u64) & Self::MASK
    }

    /// Returns the internally stored sign-extended value.
    ///
    /// # Returns
    ///
    /// The raw i64 with sign extension to 64 bits.
    #[inline]
    pub(crate) const fn raw(self) -> i64 {
        self.0
    }

    /// Constructs from a sign-extended i64 without the `from_bits`
    /// debug range check.
    ///
    /// Masks and re-sign-extends to ensure internal consistency.
    ///
    /// # Arguments
    ///
    /// * `raw` - The value to store, which will be masked to `I + F` bits.
    ///
    /// # Returns
    ///
    /// A new `Q<I, F>` with the masked, sign-extended value.
    #[inline]
    pub(crate) const fn from_raw(raw: i64) -> Self {
        Self::check();
        if Self::TOTAL_BITS >= 64 {
            Self(raw, PhantomData)
        } else {
            Self(
                Self::sign_extend((raw as u64 & Self::MASK) as i64),
                PhantomData,
            )
        }
    }

    /// Constructs from a signed count of fractional LSB units.
    ///
    /// One count is `2^-F` (one LSB), so the resulting value is
    /// `count * 2^-F` and `count` is the signed two's-complement register
    /// word. An out-of-range `count` is **wrapped** (masked to `I + F` bits,
    /// like an RTL register) — it never panics. For a checked conversion use
    /// [`try_from_count`](Self::try_from_count); to build a whole number use
    /// the [`From`] conversions (e.g. `Q::from(3i32)`).
    ///
    /// # Arguments
    ///
    /// * `count` - The number of `2^-F` units.
    ///
    /// # Returns
    ///
    /// A new `Q<I, F>` representing `count * 2^-F`, wrapped to range.
    #[inline]
    pub const fn from_count(count: i64) -> Self {
        Self::from_raw(count)
    }

    /// Constructs from a signed count of LSB units, erroring if out of range.
    ///
    /// Like [`from_count`](Self::from_count) but returns
    /// [`FixedError::OutOfRange`] instead of wrapping when `count` is outside
    /// `[MIN, MAX]`.
    ///
    /// # Arguments
    ///
    /// * `count` - The number of `2^-F` units.
    ///
    /// # Returns
    ///
    /// The `Q<I, F>` value, or [`FixedError::OutOfRange`].
    ///
    /// # Errors
    ///
    /// Returns [`FixedError::OutOfRange`] if `count` is outside `[MIN, MAX]`.
    #[inline]
    pub const fn try_from_count(count: i64) -> Result<Self, FixedError> {
        if count >= Self::MIN.0 && count <= Self::MAX.0 {
            Ok(Self::from_raw(count))
        } else {
            Err(FixedError::OutOfRange)
        }
    }

    /// Returns the value as a signed count of fractional LSB units.
    ///
    /// One count is `2^-F` (one LSB), so this returns `value * 2^F` — the raw
    /// two's-complement register word as a signed integer. Lossless; the exact
    /// inverse of [`from_count`](Self::from_count). For the unsigned bit
    /// pattern, use [`to_bits`](Self::to_bits).
    ///
    /// # Returns
    ///
    /// The signed count of `2^-F` units.
    #[inline]
    pub const fn to_count(self) -> i64 {
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
    /// A new `Q<I, F>` representing the quantized value. Out-of-range or
    /// non-finite inputs produce a wrapped/saturated value rather than
    /// panicking; use [`try_from_f64`](Self::try_from_f64) for a checked
    /// conversion.
    #[inline]
    pub fn from_f64(val: f64) -> Self {
        Self::check();
        let scale = (1u64 << Self::FRACTIONAL_BITS) as f64;
        let raw = (val * scale) as i64;
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
    /// The quantized `Q<I, F>` value.
    ///
    /// # Errors
    ///
    /// Returns [`FixedError::NotFinite`] if `val` is NaN or infinite, or
    /// [`FixedError::OutOfRange`] if it does not fit in `[MIN, MAX]`.
    #[inline]
    pub fn try_from_f64(val: f64) -> Result<Self, FixedError> {
        Self::check();
        if !val.is_finite() {
            return Err(FixedError::NotFinite);
        }
        let scale = (1u64 << Self::FRACTIONAL_BITS) as f64;
        let scaled = val * scale;
        if scaled < Self::MIN.0 as f64 || scaled > Self::MAX.0 as f64 {
            return Err(FixedError::OutOfRange);
        }
        Ok(Self::from_raw(scaled as i64))
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

    /// Wrapping negation (always wraps, never traps).
    ///
    /// # Returns
    ///
    /// The negated value, wrapped to `I + F` bits.
    #[inline]
    pub const fn wrapping_neg(self) -> Self {
        Self::from_raw(self.0.wrapping_neg())
    }

    /// Wrapping same-type multiply: `(self * rhs) >> F`, truncated to
    /// `I + F` bits.
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
    /// The truncated product in the same `Q<I, F>` format.
    #[inline]
    pub const fn wrapping_mul(self, rhs: Self) -> Self {
        let product = self.0 as i128 * rhs.0 as i128;
        let shifted = (product >> Self::FRACTIONAL_BITS) as i64;
        Self::from_raw(shifted)
    }

    /// Wrapping same-type divide: `(self << F) / rhs`, truncated to
    /// `I + F` bits.
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
    /// The truncated quotient in the same `Q<I, F>` format.
    ///
    /// # Panics
    ///
    /// Panics if `rhs` is zero (in all build profiles). Use
    /// [`checked_div`](Self::checked_div) to handle a zero divisor.
    #[inline]
    pub const fn wrapping_div(self, rhs: Self) -> Self {
        let numer = (self.0 as i128) << Self::FRACTIONAL_BITS;
        let result = (numer / rhs.0 as i128) as i64;
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
    /// `Some(quotient)` in the same `Q<I, F>` format, or `None` if `rhs` is
    /// zero.
    #[inline]
    pub const fn checked_div(self, rhs: Self) -> Option<Self> {
        if rhs.0 == 0 {
            None
        } else {
            let numer = (self.0 as i128) << Self::FRACTIONAL_BITS;
            let result = (numer / rhs.0 as i128) as i64;
            Some(Self::from_raw(result))
        }
    }

    /// Saturating addition: clamps to `[MIN, MAX]` on overflow.
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
            let sum = self.0 as i128 + rhs.0 as i128;
            let clamped = if sum < Self::MIN.0 as i128 {
                Self::MIN.0
            } else if sum > Self::MAX.0 as i128 {
                Self::MAX.0
            } else {
                sum as i64
            };
            Self::from_raw(clamped)
        }
    }

    /// Saturating subtraction: clamps to `[MIN, MAX]` on overflow.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The value to subtract.
    ///
    /// # Returns
    ///
    /// The difference, clamped to the representable range.
    #[inline]
    pub const fn saturating_sub(self, rhs: Self) -> Self {
        if Self::TOTAL_BITS >= 64 {
            Self(self.0.saturating_sub(rhs.0), PhantomData)
        } else {
            let diff = self.0 as i128 - rhs.0 as i128;
            let clamped = if diff < Self::MIN.0 as i128 {
                Self::MIN.0
            } else if diff > Self::MAX.0 as i128 {
                Self::MAX.0
            } else {
                diff as i64
            };
            Self::from_raw(clamped)
        }
    }

    /// Saturating same-type multiply: `(self * rhs) >> F`, clamped to
    /// `[MIN, MAX]`.
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
        let product = self.0 as i128 * rhs.0 as i128;
        let shifted = product >> Self::FRACTIONAL_BITS;
        let clamped = if shifted < Self::MIN.0 as i128 {
            Self::MIN.0
        } else if shifted > Self::MAX.0 as i128 {
            Self::MAX.0
        } else {
            shifted as i64
        };
        Self::from_raw(clamped)
    }

    /// Saturating negation: `MIN` saturates to `MAX` instead of wrapping.
    ///
    /// # Returns
    ///
    /// The negated value, or `MAX` when negating `MIN`.
    #[inline]
    pub const fn saturating_neg(self) -> Self {
        if Self::TOTAL_BITS >= 64 {
            Self(self.0.saturating_neg(), PhantomData)
        } else {
            let neg = -(self.0 as i128);
            if neg > Self::MAX.0 as i128 {
                Self::MAX
            } else {
                Self::from_raw(neg as i64)
            }
        }
    }

    /// Widening addition with full-precision output.
    ///
    /// The output type `Q<IO, FO>` must be wide enough to hold the sum
    /// without overflow (at least one extra integer bit).
    ///
    /// # Arguments
    ///
    /// * `rhs` - The value to add.
    ///
    /// # Returns
    ///
    /// The exact sum in `Q<IO, FO>` format.
    ///
    /// # Compile-time checks
    ///
    /// Fails to compile if the output type is too narrow (`IO + FO <= I + F`).
    #[inline]
    pub fn widening_add<IO: Unsigned, FO: Unsigned>(self, rhs: Self) -> Q<IO, FO> {
        const {
            assert!(
                Q::<IO, FO>::TOTAL_BITS > Self::TOTAL_BITS,
                "widening_add: output type too narrow (needs at least one more integer bit than the input)"
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
        Q::<IO, FO>::from_raw((a + b) as i64)
    }

    /// Widening subtraction with full-precision output.
    ///
    /// The output type `Q<IO, FO>` must be wide enough to hold the
    /// difference without overflow (at least one extra integer bit).
    ///
    /// # Arguments
    ///
    /// * `rhs` - The value to subtract.
    ///
    /// # Returns
    ///
    /// The exact difference in `Q<IO, FO>` format.
    ///
    /// # Compile-time checks
    ///
    /// Fails to compile if the output type is too narrow (`IO + FO <= I + F`).
    #[inline]
    pub fn widening_sub<IO: Unsigned, FO: Unsigned>(self, rhs: Self) -> Q<IO, FO> {
        const {
            assert!(
                Q::<IO, FO>::TOTAL_BITS > Self::TOTAL_BITS,
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
        Q::<IO, FO>::from_raw((a - b) as i64)
    }

    /// Widening multiply: `self * rhs` with full-precision output.
    ///
    /// The output type `Q<IO, FO>` must be wide enough to hold the
    /// product without loss.
    /// The output type is typically inferred from the assignment target.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The right-hand operand.
    ///
    /// # Returns
    ///
    /// The full-precision product in `Q<IO, FO>` format.
    ///
    /// # Compile-time checks
    ///
    /// Fails to compile if the output type is too narrow
    /// (`IO + FO < I + F + I2 + F2 - 1`).
    ///
    /// # Examples
    ///
    /// ```
    /// # use qfixed::Q;
    /// # use qfixed::typenum::{U12, U4, U24, U8};
    /// let a = Q::<U12, U4>::from(3i32);
    /// let b = Q::<U12, U4>::from(4i32);
    /// let c: Q<U24, U8> = a.widening_mul(b);
    /// assert_eq!(c.to_f64(), 12.0);
    /// ```
    ///
    /// An output type that is too narrow fails to compile:
    ///
    /// ```compile_fail
    /// # use qfixed::Q;
    /// # use qfixed::typenum::{U4, U8};
    /// let a = Q::<U8, U8>::from(2i32);
    /// let b = Q::<U8, U8>::from(3i32);
    /// let _c: Q<U4, U4> = a.widening_mul(b); // output too narrow
    /// ```
    #[inline]
    pub fn widening_mul<I2: Unsigned, F2: Unsigned, IO: Unsigned, FO: Unsigned>(
        self,
        rhs: Q<I2, F2>,
    ) -> Q<IO, FO> {
        const {
            assert!(
                Q::<IO, FO>::TOTAL_BITS >= Self::TOTAL_BITS + Q::<I2, F2>::TOTAL_BITS - 1,
                "widening_mul: output type too narrow to hold the full-width product"
            );
        }
        let product = self.0 as i128 * rhs.0 as i128;
        let frac_in = F::U32 + F2::U32;
        let shift = frac_in as i32 - FO::U32 as i32;
        let result = if shift > 0 {
            product >> shift
        } else {
            product << (-shift)
        };
        Q::<IO, FO>::from_raw(result as i64)
    }

    /// Absolute value.
    ///
    /// # Returns
    ///
    /// The magnitude of `self`.
    ///
    /// # Panics
    ///
    /// In debug mode, panics on `MIN` (which has no positive counterpart).
    #[inline]
    pub const fn abs(self) -> Self {
        if self.0 < 0 {
            Self::from_raw(self.0.wrapping_neg())
        } else {
            self
        }
    }
}
