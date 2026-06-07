//! Conversion operations between `Q` and `UQ` types and integer primitives.

use typenum::Unsigned;

use crate::cq::CQ;
use crate::error::FixedError;
use crate::q::Q;
use crate::uq::UQ;

// ---------------------------------------------------------------------------
// Q<I, F>: widen, truncate, saturate, reformat, to_unsigned
// ---------------------------------------------------------------------------

impl<I: Unsigned, F: Unsigned> Q<I, F> {
    /// Lossless widening to a larger `Q` type.
    ///
    /// The output must have at least as many integer and fractional bits
    /// as the input.
    ///
    /// # Returns
    ///
    /// The same value in `Q<IO, FO>` format.
    ///
    /// # Compile-time checks
    ///
    /// Fails to compile if `IO < I` or `FO < F` (the output would lose bits).
    #[inline]
    pub fn widen<IO: Unsigned, FO: Unsigned>(self) -> Q<IO, FO> {
        const {
            assert!(
                IO::U32 >= I::U32 && FO::U32 >= F::U32,
                "widen: output type must have at least as many integer and fractional bits as the input"
            );
        }
        let shift = FO::U32 as i32 - F::U32 as i32;
        let widened = if shift >= 0 {
            self.raw() << shift
        } else {
            self.raw() >> (-shift)
        };
        Q::<IO, FO>::from_raw(widened)
    }

    /// Narrowing conversion with bit truncation (floor toward −∞).
    ///
    /// Models Verilog's sized assignment: excess low bits are dropped, which
    /// for two's-complement values floors toward −∞ (e.g. `-2.5` → `-3`). See
    /// [`round_to_zero`](Self::round_to_zero) to round toward zero instead.
    ///
    /// # Returns
    ///
    /// The truncated value in `Q<IO, FO>` format.
    #[inline]
    pub fn truncate<IO: Unsigned, FO: Unsigned>(self) -> Q<IO, FO> {
        Q::<IO, FO>::check();
        let shift = F::U32 as i32 - FO::U32 as i32;
        let shifted = if shift >= 0 {
            self.raw() >> shift
        } else {
            self.raw() << (-shift)
        };
        Q::<IO, FO>::from_raw(shifted)
    }

    /// Narrowing conversion with saturation (clamp to destination range).
    ///
    /// # Returns
    ///
    /// The value clamped to fit in `Q<IO, FO>`.
    #[inline]
    pub fn saturate<IO: Unsigned, FO: Unsigned>(self) -> Q<IO, FO> {
        Q::<IO, FO>::check();
        let shift = F::U32 as i32 - FO::U32 as i32;
        let shifted = if shift >= 0 {
            self.raw() >> shift
        } else {
            self.raw() << (-shift)
        };
        let clamped = shifted.clamp(Q::<IO, FO>::MIN.raw(), Q::<IO, FO>::MAX.raw());
        Q::<IO, FO>::from_raw(clamped)
    }

    /// Value-preserving narrowing: succeeds only if the value is represented
    /// **exactly** in `Q<IO, FO>`, otherwise errors.
    ///
    /// The runtime counterpart to [`widen`](Self::widen): the number is never
    /// changed. If fractional bits would be lost it returns
    /// [`FixedError::Inexact`]; if the magnitude does not fit it returns
    /// [`FixedError::OutOfRange`]. For deliberate precision reduction use a
    /// rounding method ([`round_to_zero`](Self::round_to_zero)) or
    /// [`truncate`](Self::truncate)/[`saturate`](Self::saturate).
    ///
    /// # Returns
    ///
    /// The exact value in `Q<IO, FO>` format.
    ///
    /// # Errors
    ///
    /// Returns [`FixedError::Inexact`] if fractional precision would be lost,
    /// or [`FixedError::OutOfRange`] if the value does not fit the target's
    /// range.
    #[inline]
    pub fn try_narrow<IO: Unsigned, FO: Unsigned>(self) -> Result<Q<IO, FO>, FixedError> {
        Q::<IO, FO>::check();
        let raw = self.raw() as i128;
        let fdiff = F::U32 as i32 - FO::U32 as i32;
        let n = if fdiff > 0 {
            // Dropping `fdiff` low fractional bits — exact only if they're zero.
            if raw & ((1i128 << fdiff) - 1) != 0 {
                return Err(FixedError::Inexact);
            }
            raw >> fdiff
        } else {
            // FO >= F: adding fractional precision is always exact.
            raw << (-fdiff)
        };
        if n >= Q::<IO, FO>::MIN.raw() as i128 && n <= Q::<IO, FO>::MAX.raw() as i128 {
            Ok(Q::<IO, FO>::from_raw(n as i64))
        } else {
            Err(FixedError::OutOfRange)
        }
    }

    /// Narrowing to `Q<IO, FO>`, rounding toward zero (reducing magnitude).
    ///
    /// Drops fractional precision by rounding the magnitude down, so `-2.7`
    /// becomes `-2.0` — unlike [`truncate`](Self::truncate), which drops the
    /// low bits and floors toward −∞ to `-3.0`. (For non-negative values the
    /// two coincide.) Rounding toward zero never increases the magnitude, so it
    /// cannot overflow from a rounding carry; if the target also has fewer
    /// integer bits, an out-of-range integer part wraps, like
    /// [`truncate`](Self::truncate).
    ///
    /// # Returns
    ///
    /// The value rounded toward zero, in `Q<IO, FO>` format.
    #[inline]
    pub fn round_to_zero<IO: Unsigned, FO: Unsigned>(self) -> Q<IO, FO> {
        Q::<IO, FO>::check();
        let raw = self.raw() as i128;
        let fdiff = F::U32 as i32 - FO::U32 as i32;
        let n = if fdiff > 0 {
            // Drop `fdiff` low bits of the magnitude (toward zero for both signs).
            if raw >= 0 {
                raw >> fdiff
            } else {
                -((-raw) >> fdiff)
            }
        } else {
            raw << (-fdiff)
        };
        Q::<IO, FO>::from_raw(n as i64)
    }

    /// General-purpose format conversion that adjusts the fractional point.
    ///
    /// Use `widen` when you know the conversion is lossless, or
    /// `truncate`/`saturate` when narrowing.
    ///
    /// # Returns
    ///
    /// The value in `Q<IO, FO>` format, with fractional bits shifted
    /// accordingly.
    #[inline]
    pub fn reformat<IO: Unsigned, FO: Unsigned>(self) -> Q<IO, FO> {
        Q::<IO, FO>::check();
        let shift = FO::U32 as i32 - F::U32 as i32;
        let shifted = if shift >= 0 {
            self.raw() << shift
        } else {
            self.raw() >> (-shift)
        };
        Q::<IO, FO>::from_raw(shifted)
    }

    /// Converts signed `Q` to unsigned `UQ`, clamping negative values to
    /// zero.
    ///
    /// # Returns
    ///
    /// The value as `UQ<IO, FO>`, or `UQ::ZERO` if `self` is negative.
    #[inline]
    pub fn to_unsigned<IO: Unsigned, FO: Unsigned>(self) -> UQ<IO, FO> {
        UQ::<IO, FO>::check();
        if self.raw() < 0 {
            return UQ::<IO, FO>::ZERO;
        }
        let shift = FO::U32 as i32 - F::U32 as i32;
        let shifted = if shift >= 0 {
            (self.raw() as u64) << shift
        } else {
            (self.raw() as u64) >> (-shift)
        };
        UQ::<IO, FO>::from_raw(shifted)
    }
}

// ---------------------------------------------------------------------------
// UQ<I, F>: widen, truncate, saturate, reformat, to_signed
// ---------------------------------------------------------------------------

impl<I: Unsigned, F: Unsigned> UQ<I, F> {
    /// Lossless widening to a larger `UQ` type.
    ///
    /// # Returns
    ///
    /// The same value in `UQ<IO, FO>` format.
    ///
    /// # Compile-time checks
    ///
    /// Fails to compile if `IO < I` or `FO < F` (the output would lose bits).
    #[inline]
    pub fn widen<IO: Unsigned, FO: Unsigned>(self) -> UQ<IO, FO> {
        const {
            assert!(
                IO::U32 >= I::U32 && FO::U32 >= F::U32,
                "widen: output type must have at least as many integer and fractional bits as the input"
            );
        }
        let shift = FO::U32 as i32 - F::U32 as i32;
        let widened = if shift >= 0 {
            self.raw() << shift
        } else {
            self.raw() >> (-shift)
        };
        UQ::<IO, FO>::from_raw(widened)
    }

    /// Narrowing conversion with truncation.
    ///
    /// # Returns
    ///
    /// The truncated value in `UQ<IO, FO>` format.
    #[inline]
    pub fn truncate<IO: Unsigned, FO: Unsigned>(self) -> UQ<IO, FO> {
        UQ::<IO, FO>::check();
        let shift = F::U32 as i32 - FO::U32 as i32;
        let shifted = if shift >= 0 {
            self.raw() >> shift
        } else {
            self.raw() << (-shift)
        };
        UQ::<IO, FO>::from_raw(shifted)
    }

    /// Narrowing conversion with saturation.
    ///
    /// # Returns
    ///
    /// The value clamped to fit in `UQ<IO, FO>`.
    #[inline]
    pub fn saturate<IO: Unsigned, FO: Unsigned>(self) -> UQ<IO, FO> {
        UQ::<IO, FO>::check();
        let shift = F::U32 as i32 - FO::U32 as i32;
        let shifted = if shift >= 0 {
            self.raw() >> shift
        } else {
            self.raw() << (-shift)
        };
        let clamped = shifted.min(UQ::<IO, FO>::MAX.raw());
        UQ::<IO, FO>::from_raw(clamped)
    }

    /// Value-preserving narrowing: succeeds only if the value is represented
    /// **exactly** in `UQ<IO, FO>`, otherwise errors.
    ///
    /// The runtime counterpart to [`widen`](Self::widen): the number is never
    /// changed. If fractional bits would be lost it returns
    /// [`FixedError::Inexact`]; if the magnitude does not fit it returns
    /// [`FixedError::OutOfRange`]. For deliberate precision reduction use a
    /// rounding method ([`round_to_zero`](Self::round_to_zero)) or
    /// [`truncate`](Self::truncate)/[`saturate`](Self::saturate).
    ///
    /// # Returns
    ///
    /// The exact value in `UQ<IO, FO>` format.
    ///
    /// # Errors
    ///
    /// Returns [`FixedError::Inexact`] if fractional precision would be lost,
    /// or [`FixedError::OutOfRange`] if the value exceeds the target's maximum.
    #[inline]
    pub fn try_narrow<IO: Unsigned, FO: Unsigned>(self) -> Result<UQ<IO, FO>, FixedError> {
        UQ::<IO, FO>::check();
        let raw = self.raw() as u128;
        let fdiff = F::U32 as i32 - FO::U32 as i32;
        let n = if fdiff > 0 {
            if raw & ((1u128 << fdiff) - 1) != 0 {
                return Err(FixedError::Inexact);
            }
            raw >> fdiff
        } else {
            raw << (-fdiff)
        };
        if n <= UQ::<IO, FO>::MAX.raw() as u128 {
            Ok(UQ::<IO, FO>::from_raw(n as u64))
        } else {
            Err(FixedError::OutOfRange)
        }
    }

    /// Narrowing to `UQ<IO, FO>`, rounding toward zero.
    ///
    /// For unsigned values rounding toward zero coincides with
    /// [`truncate`](Self::truncate) (both drop the low fractional bits);
    /// provided for API symmetry with [`Q::round_to_zero`]. An out-of-range
    /// integer part wraps, like [`truncate`](Self::truncate).
    ///
    /// # Returns
    ///
    /// The value rounded toward zero, in `UQ<IO, FO>` format.
    #[inline]
    pub fn round_to_zero<IO: Unsigned, FO: Unsigned>(self) -> UQ<IO, FO> {
        UQ::<IO, FO>::check();
        let raw = self.raw() as u128;
        let fdiff = F::U32 as i32 - FO::U32 as i32;
        let n = if fdiff > 0 {
            raw >> fdiff
        } else {
            raw << (-fdiff)
        };
        UQ::<IO, FO>::from_raw(n as u64)
    }

    /// General-purpose format conversion.
    ///
    /// # Returns
    ///
    /// The value in `UQ<IO, FO>` format, with fractional bits shifted
    /// accordingly.
    #[inline]
    pub fn reformat<IO: Unsigned, FO: Unsigned>(self) -> UQ<IO, FO> {
        UQ::<IO, FO>::check();
        let shift = FO::U32 as i32 - F::U32 as i32;
        let shifted = if shift >= 0 {
            self.raw() << shift
        } else {
            self.raw() >> (-shift)
        };
        UQ::<IO, FO>::from_raw(shifted)
    }

    /// Converts unsigned `UQ` to signed `Q`.
    ///
    /// A value that does not fit the signed range is **wrapped** (masked to
    /// `IO + FO` bits) rather than panicking. For a checked conversion use
    /// [`try_to_signed`](Self::try_to_signed).
    ///
    /// # Returns
    ///
    /// The value as `Q<IO, FO>`, wrapped to range.
    #[inline]
    pub fn to_signed<IO: Unsigned, FO: Unsigned>(self) -> Q<IO, FO> {
        Q::<IO, FO>::check();
        let shift = FO::U32 as i32 - F::U32 as i32;
        let shifted = if shift >= 0 {
            (self.raw() as i64) << shift
        } else {
            (self.raw() as i64) >> (-shift)
        };
        Q::<IO, FO>::from_raw(shifted)
    }

    /// Converts unsigned `UQ` to signed `Q`, erroring if it does not fit.
    ///
    /// Like [`to_signed`](Self::to_signed) but returns
    /// [`FixedError::OutOfRange`] instead of wrapping when the value falls
    /// outside `Q<IO, FO>`'s range.
    ///
    /// # Returns
    ///
    /// The value as `Q<IO, FO>`, or [`FixedError::OutOfRange`].
    ///
    /// # Errors
    ///
    /// Returns [`FixedError::OutOfRange`] if the value does not fit the signed
    /// range of `Q<IO, FO>`.
    #[inline]
    pub fn try_to_signed<IO: Unsigned, FO: Unsigned>(self) -> Result<Q<IO, FO>, FixedError> {
        Q::<IO, FO>::check();
        let shift = FO::U32 as i32 - F::U32 as i32;
        let shifted = if shift >= 0 {
            (self.raw() as i64) << shift
        } else {
            (self.raw() as i64) >> (-shift)
        };
        if shifted >= Q::<IO, FO>::MIN.raw() && shifted <= Q::<IO, FO>::MAX.raw() {
            Ok(Q::<IO, FO>::from_raw(shifted))
        } else {
            Err(FixedError::OutOfRange)
        }
    }
}

// ---------------------------------------------------------------------------
// From<integer> for Q<I, F>
// ---------------------------------------------------------------------------

macro_rules! impl_from_signed_int {
    ($($int:ty),*) => {
        $(
            impl<I: Unsigned, F: Unsigned> From<$int> for Q<I, F> {
                /// Converts a whole number to `Q<I, F>` by shifting left by `F`.
                #[inline]
                fn from(val: $int) -> Self {
                    Self::from_count((val as i64) << Self::FRACTIONAL_BITS)
                }
            }
        )*
    };
}

impl_from_signed_int!(i8, i16, i32, i64);

// ---------------------------------------------------------------------------
// From<integer> for UQ<I, F>
// ---------------------------------------------------------------------------

macro_rules! impl_from_unsigned_int {
    ($($int:ty),*) => {
        $(
            impl<I: Unsigned, F: Unsigned> From<$int> for UQ<I, F> {
                /// Converts a whole number to `UQ<I, F>` by shifting left by `F`.
                #[inline]
                fn from(val: $int) -> Self {
                    Self::from_count((val as u64) << Self::FRACTIONAL_BITS)
                }
            }
        )*
    };
}

impl_from_unsigned_int!(u8, u16, u32, u64);

// ---------------------------------------------------------------------------
// CQ<I, F>: widen, truncate, saturate, reformat (componentwise)
// ---------------------------------------------------------------------------

impl<I: Unsigned, F: Unsigned> CQ<I, F> {
    /// Lossless widening of both components to a larger `CQ` type.
    ///
    /// # Returns
    ///
    /// The same value in `CQ<IO, FO>` format.
    ///
    /// # Compile-time checks
    ///
    /// Fails to compile if `IO < I` or `FO < F` (the output would lose bits).
    #[inline]
    pub fn widen<IO: Unsigned, FO: Unsigned>(self) -> CQ<IO, FO> {
        CQ {
            re: self.re.widen(),
            im: self.im.widen(),
        }
    }

    /// Narrowing conversion of both components with truncation (floor toward
    /// −∞); see [`round_to_zero`](Self::round_to_zero) to round toward zero.
    ///
    /// # Returns
    ///
    /// The truncated value in `CQ<IO, FO>` format.
    #[inline]
    pub fn truncate<IO: Unsigned, FO: Unsigned>(self) -> CQ<IO, FO> {
        CQ {
            re: self.re.truncate(),
            im: self.im.truncate(),
        }
    }

    /// Narrowing conversion of both components with saturation.
    ///
    /// # Returns
    ///
    /// The value with each component clamped to fit in `CQ<IO, FO>`.
    #[inline]
    pub fn saturate<IO: Unsigned, FO: Unsigned>(self) -> CQ<IO, FO> {
        CQ {
            re: self.re.saturate(),
            im: self.im.saturate(),
        }
    }

    /// Value-preserving narrowing: succeeds only if both components are
    /// represented **exactly** in `Q<IO, FO>`.
    ///
    /// Componentwise [`Q::try_narrow`].
    ///
    /// # Returns
    ///
    /// The exact value in `CQ<IO, FO>` format.
    ///
    /// # Errors
    ///
    /// Returns [`FixedError::Inexact`] if either component would lose
    /// fractional precision, or [`FixedError::OutOfRange`] if either does not
    /// fit `Q<IO, FO>`.
    #[inline]
    pub fn try_narrow<IO: Unsigned, FO: Unsigned>(self) -> Result<CQ<IO, FO>, FixedError> {
        Ok(CQ {
            re: self.re.try_narrow()?,
            im: self.im.try_narrow()?,
        })
    }

    /// Narrowing of both components, rounding toward zero.
    ///
    /// Componentwise [`Q::round_to_zero`].
    ///
    /// # Returns
    ///
    /// The value rounded toward zero, in `CQ<IO, FO>` format.
    #[inline]
    pub fn round_to_zero<IO: Unsigned, FO: Unsigned>(self) -> CQ<IO, FO> {
        CQ {
            re: self.re.round_to_zero(),
            im: self.im.round_to_zero(),
        }
    }

    /// General-purpose format conversion of both components.
    ///
    /// # Returns
    ///
    /// The value in `CQ<IO, FO>` format, with each fractional point shifted
    /// accordingly.
    #[inline]
    pub fn reformat<IO: Unsigned, FO: Unsigned>(self) -> CQ<IO, FO> {
        CQ {
            re: self.re.reformat(),
            im: self.im.reformat(),
        }
    }
}
