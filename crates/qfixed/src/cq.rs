//! Signed complex fixed-point type `CQ<I, F>`.

use core::ops::Add;

use typenum::{Sum, U1, Unsigned};

use crate::q::Q;

/// Signed complex fixed-point number: a real and imaginary `Q<I, F>` pair.
///
/// `I` and `F` are [`typenum`](crate::typenum) unsigned integers, e.g.
/// `CQ<U4, U12>`. Both components share the same `Q<I, F>` format, so the
/// total storage is two `i64` registers regardless of bit width — ideal for
/// bit-accurate modeling of IQ-sample and complex datapaths, but not a packed
/// wire format.
///
/// Complex numbers have no total order, so `CQ` deliberately does **not**
/// implement `Ord`/`PartialOrd` (unlike [`Q`](crate::Q)).
///
/// # Bit growth
///
/// Add/sub grow like the scalar [`Q`](crate::Q) (one extra integer bit).
/// Complex multiply grows by **one integer bit more than a real multiply**,
/// because each output component is a sum or difference of two full products:
/// `(a + bi)(c + di) = (ac - bd) + (ad + bc)i`. So
/// `CQ<I, F> * CQ<I, F>` yields `CQ<2*I + 1, 2*F>`.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct CQ<I, F> {
    /// The real part.
    pub re: Q<I, F>,

    /// The imaginary part.
    pub im: Q<I, F>,
}

impl<I: Unsigned, F: Unsigned> CQ<I, F> {
    /// Total number of integer bits in each component of this type.
    pub const INTEGER_BITS: u32 = I::U32;

    /// Total number of fractional bits in each component of this type.
    pub const FRACTIONAL_BITS: u32 = F::U32;

    /// Total number of bits in each component of this type.
    pub const TOTAL_BITS: u32 = Self::INTEGER_BITS + Self::FRACTIONAL_BITS;

    /// The value zero (`0 + 0i`).
    pub const ZERO: Self = Self {
        re: Q::<I, F>::ZERO,
        im: Q::<I, F>::ZERO,
    };

    /// The real unit (`1 + 0i`).
    ///
    /// Only available when `I > 1` (at least one non-sign integer bit).
    ///
    /// # Panics
    ///
    /// Panics at compile time if `I <= 1` (inherited from [`Q::ONE`]).
    pub const ONE: Self = Self {
        re: Q::<I, F>::ONE,
        im: Q::<I, F>::ZERO,
    };

    /// The imaginary unit (`0 + 1i`).
    ///
    /// Named `J` after the electrical-engineering convention (and to avoid
    /// clashing with the `I` integer-bits parameter).
    ///
    /// # Panics
    ///
    /// Panics at compile time if `I <= 1` (inherited from [`Q::ONE`]).
    pub const J: Self = Self {
        re: Q::<I, F>::ZERO,
        im: Q::<I, F>::ONE,
    };

    /// Constructs from real and imaginary components.
    ///
    /// # Arguments
    ///
    /// * `re` - The real part.
    /// * `im` - The imaginary part.
    ///
    /// # Returns
    ///
    /// A new `CQ<I, F>` with the given components.
    #[inline]
    pub const fn new(re: Q<I, F>, im: Q<I, F>) -> Self {
        Self { re, im }
    }

    /// Constructs a purely real value (imaginary part zero).
    ///
    /// # Arguments
    ///
    /// * `re` - The real part.
    ///
    /// # Returns
    ///
    /// A new `CQ<I, F>` with `im = 0`.
    #[inline]
    pub const fn from_re(re: Q<I, F>) -> Self {
        Self {
            re,
            im: Q::<I, F>::ZERO,
        }
    }

    /// Constructs from the raw bit patterns of each component, sign-extending.
    ///
    /// Delegates to [`Q::from_bits`], so each argument is the unsigned
    /// two's-complement pattern produced by [`to_bits`](Self::to_bits), and
    /// this is its exact inverse (including for negative components).
    ///
    /// # Arguments
    ///
    /// * `re` - The raw real-part bit pattern.
    /// * `im` - The raw imaginary-part bit pattern.
    ///
    /// # Returns
    ///
    /// A new `CQ<I, F>` holding the sign-extended values.
    ///
    /// # Panics
    ///
    /// In debug mode, panics if either value carries information outside the
    /// low `I + F` bits (see [`Q::from_bits`]).
    #[inline]
    pub const fn from_bits(re: u64, im: u64) -> Self {
        Self {
            re: Q::<I, F>::from_bits(re),
            im: Q::<I, F>::from_bits(im),
        }
    }

    /// Extracts the raw bit patterns of both components.
    ///
    /// # Returns
    ///
    /// A `(re, im)` tuple of the unsigned two's-complement patterns. Only the
    /// low `I + F` bits of each are meaningful.
    #[inline]
    pub const fn to_bits(self) -> (u64, u64) {
        (self.re.to_bits(), self.im.to_bits())
    }

    /// Constructs from signed counts of fractional LSB units for each
    /// component.
    ///
    /// One count is `2^-F`; each component is the signed register word.
    /// Lossless; the exact inverse of [`to_count`](Self::to_count).
    ///
    /// # Arguments
    ///
    /// * `re` - The real-part count of `2^-F` units.
    /// * `im` - The imaginary-part count of `2^-F` units.
    ///
    /// # Returns
    ///
    /// A new `CQ<I, F>` with the given component counts.
    ///
    /// # Panics
    ///
    /// In debug mode, panics if either count is outside `[MIN, MAX]`.
    #[inline]
    pub const fn from_count(re: i64, im: i64) -> Self {
        Self {
            re: Q::<I, F>::from_count(re),
            im: Q::<I, F>::from_count(im),
        }
    }

    /// Returns the signed counts of fractional LSB units for both components.
    ///
    /// # Returns
    ///
    /// A `(re, im)` tuple of signed `2^-F`-unit counts. Lossless; the exact
    /// inverse of [`from_count`](Self::from_count).
    #[inline]
    pub const fn to_count(self) -> (i64, i64) {
        (self.re.to_count(), self.im.to_count())
    }

    /// Constructs from `f64` components, quantizing each to the nearest
    /// representable value.
    ///
    /// # Arguments
    ///
    /// * `re` - The real part.
    /// * `im` - The imaginary part.
    ///
    /// # Returns
    ///
    /// A new `CQ<I, F>` representing the quantized value.
    #[inline]
    pub fn from_f64(re: f64, im: f64) -> Self {
        Self {
            re: Q::<I, F>::from_f64(re),
            im: Q::<I, F>::from_f64(im),
        }
    }

    /// Wrapping addition (componentwise, always wraps).
    ///
    /// # Arguments
    ///
    /// * `rhs` - The value to add.
    ///
    /// # Returns
    ///
    /// The sum, each component wrapped to `I + F` bits.
    #[inline]
    pub const fn wrapping_add(self, rhs: Self) -> Self {
        Self {
            re: self.re.wrapping_add(rhs.re),
            im: self.im.wrapping_add(rhs.im),
        }
    }

    /// Wrapping subtraction (componentwise, always wraps).
    ///
    /// # Arguments
    ///
    /// * `rhs` - The value to subtract.
    ///
    /// # Returns
    ///
    /// The difference, each component wrapped to `I + F` bits.
    #[inline]
    pub const fn wrapping_sub(self, rhs: Self) -> Self {
        Self {
            re: self.re.wrapping_sub(rhs.re),
            im: self.im.wrapping_sub(rhs.im),
        }
    }

    /// Wrapping negation (componentwise, always wraps).
    ///
    /// # Returns
    ///
    /// The negated value, each component wrapped to `I + F` bits.
    #[inline]
    pub const fn wrapping_neg(self) -> Self {
        Self {
            re: self.re.wrapping_neg(),
            im: self.im.wrapping_neg(),
        }
    }

    /// Wrapping same-type complex multiply, truncated to `I + F` bits.
    ///
    /// Computes `(ac - bd) + (ad + bc)i` in 128 bits, then realigns each
    /// component by shifting right by `F`.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The value to multiply by.
    ///
    /// # Returns
    ///
    /// The truncated product in the same `CQ<I, F>` format.
    #[inline]
    pub const fn wrapping_mul(self, rhs: Self) -> Self {
        let re = self.re.raw() as i128 * rhs.re.raw() as i128
            - self.im.raw() as i128 * rhs.im.raw() as i128;
        let im = self.re.raw() as i128 * rhs.im.raw() as i128
            + self.im.raw() as i128 * rhs.re.raw() as i128;
        Self {
            re: Q::<I, F>::from_raw((re >> Self::FRACTIONAL_BITS) as i64),
            im: Q::<I, F>::from_raw((im >> Self::FRACTIONAL_BITS) as i64),
        }
    }

    /// Wrapping conjugate (same width): negates the imaginary part.
    ///
    /// Wraps on the `MIN` imaginary part; use [`conj`](Self::conj) for the
    /// widening, overflow-free conjugate.
    ///
    /// # Returns
    ///
    /// The complex conjugate in the same `CQ<I, F>` format.
    #[inline]
    pub const fn wrapping_conj(self) -> Self {
        Self {
            re: self.re,
            im: self.im.wrapping_neg(),
        }
    }

    /// Saturating addition (componentwise, clamps to `[MIN, MAX]`).
    ///
    /// # Arguments
    ///
    /// * `rhs` - The value to add.
    ///
    /// # Returns
    ///
    /// The sum, each component clamped to the representable range.
    #[inline]
    pub const fn saturating_add(self, rhs: Self) -> Self {
        Self {
            re: self.re.saturating_add(rhs.re),
            im: self.im.saturating_add(rhs.im),
        }
    }

    /// Saturating subtraction (componentwise, clamps to `[MIN, MAX]`).
    ///
    /// # Arguments
    ///
    /// * `rhs` - The value to subtract.
    ///
    /// # Returns
    ///
    /// The difference, each component clamped to the representable range.
    #[inline]
    pub const fn saturating_sub(self, rhs: Self) -> Self {
        Self {
            re: self.re.saturating_sub(rhs.re),
            im: self.im.saturating_sub(rhs.im),
        }
    }

    /// Saturating same-type complex multiply, clamped to `[MIN, MAX]`.
    ///
    /// Computes `(ac - bd) + (ad + bc)i` in 128 bits and clamps each
    /// component rather than truncating.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The value to multiply by.
    ///
    /// # Returns
    ///
    /// The product with each component clamped to the representable range.
    #[inline]
    pub const fn saturating_mul(self, rhs: Self) -> Self {
        let re = (self.re.raw() as i128 * rhs.re.raw() as i128
            - self.im.raw() as i128 * rhs.im.raw() as i128)
            >> Self::FRACTIONAL_BITS;
        let im = (self.re.raw() as i128 * rhs.im.raw() as i128
            + self.im.raw() as i128 * rhs.re.raw() as i128)
            >> Self::FRACTIONAL_BITS;
        let lo = Q::<I, F>::MIN.raw() as i128;
        let hi = Q::<I, F>::MAX.raw() as i128;
        let re = if re < lo {
            lo
        } else if re > hi {
            hi
        } else {
            re
        };
        let im = if im < lo {
            lo
        } else if im > hi {
            hi
        } else {
            im
        };
        Self {
            re: Q::<I, F>::from_raw(re as i64),
            im: Q::<I, F>::from_raw(im as i64),
        }
    }

    /// Complex conjugate with full-precision output: `a - bi`.
    ///
    /// Widens by one integer bit so negating the imaginary part can never
    /// overflow (mirrors [`Neg`](core::ops::Neg) for [`Q`](crate::Q)). Use
    /// [`wrapping_conj`](Self::wrapping_conj) for same-width RTL semantics.
    ///
    /// # Returns
    ///
    /// The conjugate as `CQ<I + 1, F>`.
    #[inline]
    pub fn conj(self) -> CQ<Sum<I, U1>, F>
    where
        I: Add<U1>,
        Sum<I, U1>: Unsigned,
    {
        CQ {
            re: self.re.widen(),
            im: -self.im,
        }
    }

    /// Multiply by the imaginary unit (`+90°` rotation): `(a + bi)i = -b + ai`.
    ///
    /// Widens by one integer bit so the rotation can never overflow.
    ///
    /// # Returns
    ///
    /// `self * i` as `CQ<I + 1, F>`.
    #[inline]
    pub fn mul_j(self) -> CQ<Sum<I, U1>, F>
    where
        I: Add<U1>,
        Sum<I, U1>: Unsigned,
    {
        CQ {
            re: -self.im,
            im: self.re.widen(),
        }
    }

    /// Multiply by the negative imaginary unit (`-90°` rotation):
    /// `(a + bi)(-i) = b - ai`.
    ///
    /// Widens by one integer bit so the rotation can never overflow.
    ///
    /// # Returns
    ///
    /// `self * (-i)` as `CQ<I + 1, F>`.
    #[inline]
    pub fn mul_neg_j(self) -> CQ<Sum<I, U1>, F>
    where
        I: Add<U1>,
        Sum<I, U1>: Unsigned,
    {
        CQ {
            re: self.im.widen(),
            im: -self.re,
        }
    }

    /// Squared magnitude `re*re + im*im`, with full-precision real output.
    ///
    /// Avoids the square root, so the result stays exactly in fixed point.
    /// The output grows like a complex multiply: two full `2*F`-fractional
    /// products summed need one extra integer bit.
    ///
    /// # Returns
    ///
    /// `|self|^2` as a real `Q<2*I + 1, 2*F>`.
    #[inline]
    pub fn norm_sqr(self) -> Q<Sum<Sum<I, I>, U1>, Sum<F, F>>
    where
        I: Add<I>,
        Sum<I, I>: Add<U1> + Unsigned,
        Sum<Sum<I, I>, U1>: Unsigned,
        F: Add<F>,
        Sum<F, F>: Unsigned,
    {
        let real = self.re.raw() as i128;
        let imag = self.im.raw() as i128;
        let sum = real * real + imag * imag;
        Q::<Sum<Sum<I, I>, U1>, Sum<F, F>>::from_raw(sum as i64)
    }
}
