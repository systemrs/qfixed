//! `core::ops` trait implementations for `Q` and `UQ`.
//!
//! Implements `Add`, `Sub`, `Neg` (Q only), `Mul`, `Shl`, `Shr`, `BitAnd`,
//! `BitOr`, and [`core::iter::Sum`].
//!
//! Reducing many values with `Add` grows one integer bit *per* operation
//! (and changes the type each step). [`core::iter::Sum`] instead accumulates
//! into a single caller-chosen output type, so summing `N` values needs only
//! `ceil(log2(N))` extra integer bits — e.g. 64 values of `Q<I, F>` fit in
//! `Q<I + 6, F>`, not `Q<I + 64, F>`:
//!
//! ```
//! use qfixed::Q;
//! use qfixed::typenum::{U4, U12, U18};
//!
//! let xs = [Q::<U12, U4>::from(1000i32); 64];
//! // 64 values → 6 extra integer bits; the 128-bit accumulator can't overflow.
//! let total: Q<U18, U4> = xs.iter().copied().sum();
//! assert_eq!(total.to_f64(), 64_000.0);
//! ```
//!
//! Because the accumulator width is caller-chosen while `N` is a runtime
//! quantity, the type system cannot prove it is wide enough; an undersized
//! output wraps (RTL register semantics). Use the `try_sum` associated function
//! (`Q::try_sum` / `UQ::try_sum` / `CQ::try_sum`) for a checked reduction that
//! returns [`FixedError::OutOfRange`](crate::FixedError) instead of wrapping.
//!
//! Arithmetic operators produce widened outputs that cannot overflow. The
//! output widths are derived at the type level with [`typenum`](crate::typenum)
//! (`Sum<I, U1>` = `I + 1`, `Sum<I, I>` = `2 * I`):
//! - `Q<I,F> + Q<I,F>` → `Q<Sum<I, U1>, F>` (one extra integer bit)
//! - `Q<I,F> - Q<I,F>` → `Q<Sum<I, U1>, F>` (one extra integer bit)
//! - `Q<I,F> * Q<I,F>` → `Q<Sum<I, I>, Sum<F, F>>` (full-width product)
//! - `-Q<I,F>` → `Q<Sum<I, U1>, F>` (handles MIN)
//!
//! Use `.truncate()` or `.saturate()` to narrow the result back down.
//! Use `wrapping_add`/`wrapping_sub`/`wrapping_mul` for same-type RTL
//! truncation semantics.

// Widening operators return a wider type than `Self`, which clippy
// misinterprets as wrong-operation bugs.
#![allow(clippy::suspicious_arithmetic_impl)]

use core::ops;

use typenum::{Sum, U1, Unsigned};

use crate::cq::CQ;
use crate::error::FixedError;
use crate::q::Q;
use crate::uq::UQ;

// ===========================================================================
// Q<I, F>
// ===========================================================================

// ---------------------------------------------------------------------------
// Q<I, F>: Add
// ---------------------------------------------------------------------------

impl<I, F> ops::Add for Q<I, F>
where
    I: Unsigned + ops::Add<U1>,
    F: Unsigned,
    Sum<I, U1>: Unsigned,
{
    type Output = Q<Sum<I, U1>, F>;

    /// Adds two `Q<I, F>` values, producing `Q<Sum<I, U1>, F>`.
    ///
    /// The extra integer bit guarantees no overflow.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The right-hand operand.
    ///
    /// # Returns
    ///
    /// The exact sum as `Q<Sum<I, U1>, F>`.
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Q::<Sum<I, U1>, F>::from_raw(self.raw() + rhs.raw())
    }
}

// ---------------------------------------------------------------------------
// Q<I, F>: Sub
// ---------------------------------------------------------------------------

impl<I, F> ops::Sub for Q<I, F>
where
    I: Unsigned + ops::Add<U1>,
    F: Unsigned,
    Sum<I, U1>: Unsigned,
{
    type Output = Q<Sum<I, U1>, F>;

    /// Subtracts two `Q<I, F>` values, producing `Q<Sum<I, U1>, F>`.
    ///
    /// The extra integer bit guarantees no overflow.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The right-hand operand.
    ///
    /// # Returns
    ///
    /// The exact difference as `Q<Sum<I, U1>, F>`.
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Q::<Sum<I, U1>, F>::from_raw(self.raw() - rhs.raw())
    }
}

// ---------------------------------------------------------------------------
// Q<I, F>: Neg
// ---------------------------------------------------------------------------

impl<I, F> ops::Neg for Q<I, F>
where
    I: Unsigned + ops::Add<U1>,
    F: Unsigned,
    Sum<I, U1>: Unsigned,
{
    type Output = Q<Sum<I, U1>, F>;

    /// Negates a `Q<I, F>` value, producing `Q<Sum<I, U1>, F>`.
    ///
    /// The extra integer bit handles the MIN case without wrapping.
    ///
    /// # Returns
    ///
    /// The exact negation as `Q<Sum<I, U1>, F>`.
    #[inline]
    fn neg(self) -> Self::Output {
        Q::<Sum<I, U1>, F>::from_raw(-self.raw())
    }
}

// ---------------------------------------------------------------------------
// Q<I, F>: Mul
// ---------------------------------------------------------------------------

impl<I, F> ops::Mul for Q<I, F>
where
    I: Unsigned + ops::Add<I>,
    F: Unsigned + ops::Add<F>,
    Sum<I, I>: Unsigned,
    Sum<F, F>: Unsigned,
{
    type Output = Q<Sum<I, I>, Sum<F, F>>;

    /// Multiplies two `Q<I, F>` values, producing `Q<Sum<I, I>, Sum<F, F>>`.
    ///
    /// The full-width product cannot overflow.
    /// Use `.truncate()` or `.saturate()` to narrow the result.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The right-hand operand.
    ///
    /// # Returns
    ///
    /// The exact product as `Q<Sum<I, I>, Sum<F, F>>`.
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        let product = self.raw() as i128 * rhs.raw() as i128;
        Q::<Sum<I, I>, Sum<F, F>>::from_raw(product as i64)
    }
}

// ---------------------------------------------------------------------------
// Q<I, F>: Shl, Shr
// ---------------------------------------------------------------------------

impl<I: Unsigned, F: Unsigned> ops::Shl<u32> for Q<I, F> {
    type Output = Self;

    /// Shifts left by `shift` bits, masking the result to `I + F` bits.
    ///
    /// # Arguments
    ///
    /// * `shift` - Number of bits to shift left.
    ///
    /// # Returns
    ///
    /// The shifted value, masked to the bit width.
    #[inline]
    fn shl(self, shift: u32) -> Self {
        Self::from_raw(self.raw() << shift)
    }
}

impl<I: Unsigned, F: Unsigned> ops::ShlAssign<u32> for Q<I, F> {
    /// Shifts left in place.
    ///
    /// # Arguments
    ///
    /// * `shift` - Number of bits to shift left.
    #[inline]
    fn shl_assign(&mut self, shift: u32) {
        *self = *self << shift;
    }
}

impl<I: Unsigned, F: Unsigned> ops::Shr<u32> for Q<I, F> {
    type Output = Self;

    /// Arithmetic right shift (sign-preserving) by `shift` bits.
    ///
    /// # Arguments
    ///
    /// * `shift` - Number of bits to shift right.
    ///
    /// # Returns
    ///
    /// The shifted value with sign extension.
    #[inline]
    fn shr(self, shift: u32) -> Self {
        Self::from_raw(self.raw() >> shift)
    }
}

impl<I: Unsigned, F: Unsigned> ops::ShrAssign<u32> for Q<I, F> {
    /// Shifts right in place (arithmetic / sign-preserving).
    ///
    /// # Arguments
    ///
    /// * `shift` - Number of bits to shift right.
    #[inline]
    fn shr_assign(&mut self, shift: u32) {
        *self = *self >> shift;
    }
}

// ---------------------------------------------------------------------------
// Q<I, F>: BitAnd, BitOr
// ---------------------------------------------------------------------------

impl<I: Unsigned, F: Unsigned> ops::BitAnd for Q<I, F> {
    type Output = Self;

    /// Bitwise AND of two `Q<I, F>` values.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The right-hand operand.
    ///
    /// # Returns
    ///
    /// The bitwise AND result.
    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self::from_raw(self.raw() & rhs.raw())
    }
}

impl<I: Unsigned, F: Unsigned> ops::BitOr for Q<I, F> {
    type Output = Self;

    /// Bitwise OR of two `Q<I, F>` values.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The right-hand operand.
    ///
    /// # Returns
    ///
    /// The bitwise OR result.
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self::from_raw(self.raw() | rhs.raw())
    }
}

// ---------------------------------------------------------------------------
// Q<I, F>: Sum (logarithmic bit growth into a caller-chosen accumulator)
// ---------------------------------------------------------------------------

impl<I, F, IO, FO> core::iter::Sum<Q<I, F>> for Q<IO, FO>
where
    I: Unsigned,
    F: Unsigned,
    IO: Unsigned,
    FO: Unsigned,
{
    /// Sums an iterator of `Q<I, F>` into a caller-chosen accumulator
    /// `Q<IO, FO>`.
    ///
    /// Unlike chaining `Add` (one extra integer bit per operation), the
    /// accumulator width is fixed by the output type, so summing `N` values
    /// needs only `ceil(log2(N))` extra integer bits over the input — e.g. 64
    /// values of `Q<I, F>` fit in `Q<I + 6, F>`. Partial sums accumulate in 128
    /// bits, so no intermediate overflow occurs; choose `IO` at least
    /// `I + ceil(log2(N))` or the result wraps to `IO + FO` bits (RTL register
    /// semantics). An empty iterator yields [`Q::ZERO`]. When `FO != F` the
    /// fractional point is realigned by shifting, matching
    /// [`reformat`](Q::reformat).
    #[inline]
    fn sum<It: Iterator<Item = Q<I, F>>>(iter: It) -> Self {
        let acc: i128 = iter.map(|q| q.raw() as i128).sum();
        let shift = FO::U32 as i32 - F::U32 as i32;
        let scaled = if shift >= 0 {
            acc << shift
        } else {
            acc >> (-shift)
        };
        Q::<IO, FO>::from_raw(scaled as i64)
    }
}

impl<'a, I, F, IO, FO> core::iter::Sum<&'a Q<I, F>> for Q<IO, FO>
where
    I: Unsigned,
    F: Unsigned,
    IO: Unsigned,
    FO: Unsigned,
{
    /// Sums an iterator of `&Q<I, F>` into a caller-chosen accumulator
    /// `Q<IO, FO>`; see the owned `Sum` impl for the bit-growth contract.
    #[inline]
    fn sum<It: Iterator<Item = &'a Q<I, F>>>(iter: It) -> Self {
        iter.copied().sum()
    }
}

impl<IO: Unsigned, FO: Unsigned> Q<IO, FO> {
    /// Checked sum: accumulates an iterator of `Q<I, F>` into `Q<IO, FO>`,
    /// erroring if the exact total does not fit the chosen output type.
    ///
    /// The checked counterpart to the [`Sum`](core::iter::Sum) impl. The
    /// accumulation is identical (128-bit partial sums; fractional point
    /// realigned by shifting when `FO != F`), but instead of wrapping it
    /// returns [`FixedError::OutOfRange`] when the result falls outside
    /// `[MIN, MAX]` — letting the caller detect an under-sized accumulator
    /// rather than getting a silently masked value. An empty iterator yields
    /// `Ok(`[`ZERO`](Q::ZERO)`)`.
    ///
    /// # Errors
    ///
    /// Returns [`FixedError::OutOfRange`] if the total does not fit `Q<IO, FO>`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use qfixed::Q;
    /// # use qfixed::typenum::{U4, U12, U18};
    /// let xs = [Q::<U12, U4>::from(1000i32); 64];
    /// // Wide enough: 64 values need 6 extra integer bits.
    /// assert_eq!(Q::<U18, U4>::try_sum(xs).unwrap().to_f64(), 64_000.0);
    /// // Too narrow: the same-as-input type cannot hold the total.
    /// assert!(Q::<U12, U4>::try_sum(xs).is_err());
    /// ```
    #[inline]
    pub fn try_sum<I, F, It>(iter: It) -> Result<Self, FixedError>
    where
        I: Unsigned,
        F: Unsigned,
        It: IntoIterator<Item = Q<I, F>>,
    {
        let acc: i128 = iter.into_iter().map(|q| q.raw() as i128).sum();
        let shift = FO::U32 as i32 - F::U32 as i32;
        let scaled = if shift >= 0 {
            acc << shift
        } else {
            acc >> (-shift)
        };
        if scaled >= Q::<IO, FO>::MIN.raw() as i128 && scaled <= Q::<IO, FO>::MAX.raw() as i128 {
            Ok(Q::<IO, FO>::from_raw(scaled as i64))
        } else {
            Err(FixedError::OutOfRange)
        }
    }
}

// ===========================================================================
// UQ<I, F>
// ===========================================================================

// ---------------------------------------------------------------------------
// UQ<I, F>: Add
// ---------------------------------------------------------------------------

impl<I, F> ops::Add for UQ<I, F>
where
    I: Unsigned + ops::Add<U1>,
    F: Unsigned,
    Sum<I, U1>: Unsigned,
{
    type Output = UQ<Sum<I, U1>, F>;

    /// Adds two `UQ<I, F>` values, producing `UQ<Sum<I, U1>, F>`.
    ///
    /// The extra integer bit guarantees no overflow.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The right-hand operand.
    ///
    /// # Returns
    ///
    /// The exact sum as `UQ<Sum<I, U1>, F>`.
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        UQ::<Sum<I, U1>, F>::from_raw(self.raw() + rhs.raw())
    }
}

// ---------------------------------------------------------------------------
// UQ<I, F>: Sub → signed Q output
// ---------------------------------------------------------------------------

impl<I, F> ops::Sub for UQ<I, F>
where
    I: Unsigned + ops::Add<U1>,
    F: Unsigned,
    Sum<I, U1>: Unsigned,
{
    type Output = Q<Sum<I, U1>, F>;

    /// Subtracts two `UQ<I, F>` values, producing signed `Q<Sum<I, U1>, F>`.
    ///
    /// The result is signed because `UQ - UQ` can be negative.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The right-hand operand.
    ///
    /// # Returns
    ///
    /// The exact difference as signed `Q<Sum<I, U1>, F>`.
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Q::<Sum<I, U1>, F>::from_raw(self.raw() as i64 - rhs.raw() as i64)
    }
}

// ---------------------------------------------------------------------------
// UQ<I, F>: Mul
// ---------------------------------------------------------------------------

impl<I, F> ops::Mul for UQ<I, F>
where
    I: Unsigned + ops::Add<I>,
    F: Unsigned + ops::Add<F>,
    Sum<I, I>: Unsigned,
    Sum<F, F>: Unsigned,
{
    type Output = UQ<Sum<I, I>, Sum<F, F>>;

    /// Multiplies two `UQ<I, F>` values, producing `UQ<Sum<I, I>, Sum<F, F>>`.
    ///
    /// The full-width product cannot overflow.
    /// Use `.truncate()` or `.saturate()` to narrow the result.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The right-hand operand.
    ///
    /// # Returns
    ///
    /// The exact product as `UQ<Sum<I, I>, Sum<F, F>>`.
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        let product = self.raw() as u128 * rhs.raw() as u128;
        UQ::<Sum<I, I>, Sum<F, F>>::from_raw(product as u64)
    }
}

// ---------------------------------------------------------------------------
// UQ<I, F>: Shl, Shr
// ---------------------------------------------------------------------------

impl<I: Unsigned, F: Unsigned> ops::Shl<u32> for UQ<I, F> {
    type Output = Self;

    /// Shifts left by `shift` bits, masking the result to `I + F` bits.
    ///
    /// # Arguments
    ///
    /// * `shift` - Number of bits to shift left.
    ///
    /// # Returns
    ///
    /// The shifted value, masked to the bit width.
    #[inline]
    fn shl(self, shift: u32) -> Self {
        Self::from_raw(self.raw() << shift)
    }
}

impl<I: Unsigned, F: Unsigned> ops::ShlAssign<u32> for UQ<I, F> {
    /// Shifts left in place.
    ///
    /// # Arguments
    ///
    /// * `shift` - Number of bits to shift left.
    #[inline]
    fn shl_assign(&mut self, shift: u32) {
        *self = *self << shift;
    }
}

impl<I: Unsigned, F: Unsigned> ops::Shr<u32> for UQ<I, F> {
    type Output = Self;

    /// Logical right shift (zero-fill) by `shift` bits.
    ///
    /// # Arguments
    ///
    /// * `shift` - Number of bits to shift right.
    ///
    /// # Returns
    ///
    /// The shifted value with zero-fill.
    #[inline]
    fn shr(self, shift: u32) -> Self {
        Self::from_raw(self.raw() >> shift)
    }
}

impl<I: Unsigned, F: Unsigned> ops::ShrAssign<u32> for UQ<I, F> {
    /// Shifts right in place (logical / zero-fill).
    ///
    /// # Arguments
    ///
    /// * `shift` - Number of bits to shift right.
    #[inline]
    fn shr_assign(&mut self, shift: u32) {
        *self = *self >> shift;
    }
}

// ---------------------------------------------------------------------------
// UQ<I, F>: BitAnd, BitOr
// ---------------------------------------------------------------------------

impl<I: Unsigned, F: Unsigned> ops::BitAnd for UQ<I, F> {
    type Output = Self;

    /// Bitwise AND of two `UQ<I, F>` values.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The right-hand operand.
    ///
    /// # Returns
    ///
    /// The bitwise AND result.
    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self::from_raw(self.raw() & rhs.raw())
    }
}

impl<I: Unsigned, F: Unsigned> ops::BitOr for UQ<I, F> {
    type Output = Self;

    /// Bitwise OR of two `UQ<I, F>` values.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The right-hand operand.
    ///
    /// # Returns
    ///
    /// The bitwise OR result.
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self::from_raw(self.raw() | rhs.raw())
    }
}

// ---------------------------------------------------------------------------
// UQ<I, F>: Sum (logarithmic bit growth into a caller-chosen accumulator)
// ---------------------------------------------------------------------------

impl<I, F, IO, FO> core::iter::Sum<UQ<I, F>> for UQ<IO, FO>
where
    I: Unsigned,
    F: Unsigned,
    IO: Unsigned,
    FO: Unsigned,
{
    /// Sums an iterator of `UQ<I, F>` into a caller-chosen accumulator
    /// `UQ<IO, FO>`.
    ///
    /// Like the signed [`Q`] impl, the accumulator width is fixed by the output
    /// type, so summing `N` values needs only `ceil(log2(N))` extra integer
    /// bits over the input. Partial sums accumulate in 128 bits, so no
    /// intermediate overflow occurs; choose `IO` at least `I + ceil(log2(N))`
    /// or the result wraps to `IO + FO` bits. An empty iterator yields
    /// [`UQ::ZERO`].
    #[inline]
    fn sum<It: Iterator<Item = UQ<I, F>>>(iter: It) -> Self {
        let acc: u128 = iter.map(|q| q.raw() as u128).sum();
        let shift = FO::U32 as i32 - F::U32 as i32;
        let scaled = if shift >= 0 {
            acc << shift
        } else {
            acc >> (-shift)
        };
        UQ::<IO, FO>::from_raw(scaled as u64)
    }
}

impl<'a, I, F, IO, FO> core::iter::Sum<&'a UQ<I, F>> for UQ<IO, FO>
where
    I: Unsigned,
    F: Unsigned,
    IO: Unsigned,
    FO: Unsigned,
{
    /// Sums an iterator of `&UQ<I, F>` into a caller-chosen accumulator
    /// `UQ<IO, FO>`; see the owned `Sum` impl for the bit-growth contract.
    #[inline]
    fn sum<It: Iterator<Item = &'a UQ<I, F>>>(iter: It) -> Self {
        iter.copied().sum()
    }
}

impl<IO: Unsigned, FO: Unsigned> UQ<IO, FO> {
    /// Checked sum: accumulates an iterator of `UQ<I, F>` into `UQ<IO, FO>`,
    /// erroring if the exact total does not fit the chosen output type.
    ///
    /// The checked counterpart to the [`Sum`](core::iter::Sum) impl: identical
    /// 128-bit accumulation, but returns [`FixedError::OutOfRange`] instead of
    /// wrapping when the result exceeds `MAX`. An empty iterator yields
    /// `Ok(`[`ZERO`](UQ::ZERO)`)`.
    ///
    /// # Errors
    ///
    /// Returns [`FixedError::OutOfRange`] if the total exceeds `UQ<IO, FO>::MAX`.
    #[inline]
    pub fn try_sum<I, F, It>(iter: It) -> Result<Self, FixedError>
    where
        I: Unsigned,
        F: Unsigned,
        It: IntoIterator<Item = UQ<I, F>>,
    {
        let acc: u128 = iter.into_iter().map(|q| q.raw() as u128).sum();
        let shift = FO::U32 as i32 - F::U32 as i32;
        let scaled = if shift >= 0 {
            acc << shift
        } else {
            acc >> (-shift)
        };
        if scaled <= UQ::<IO, FO>::MAX.raw() as u128 {
            Ok(UQ::<IO, FO>::from_raw(scaled as u64))
        } else {
            Err(FixedError::OutOfRange)
        }
    }
}

// ===========================================================================
// CQ<I, F>
// ===========================================================================

// ---------------------------------------------------------------------------
// CQ<I, F>: Add
// ---------------------------------------------------------------------------

impl<I, F> ops::Add for CQ<I, F>
where
    I: Unsigned + ops::Add<U1>,
    F: Unsigned,
    Sum<I, U1>: Unsigned,
{
    type Output = CQ<Sum<I, U1>, F>;

    /// Adds two `CQ<I, F>` values, producing `CQ<Sum<I, U1>, F>`.
    ///
    /// The extra integer bit guarantees no overflow.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The right-hand operand.
    ///
    /// # Returns
    ///
    /// The exact sum as `CQ<Sum<I, U1>, F>`.
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        CQ {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}

// ---------------------------------------------------------------------------
// CQ<I, F>: Sub
// ---------------------------------------------------------------------------

impl<I, F> ops::Sub for CQ<I, F>
where
    I: Unsigned + ops::Add<U1>,
    F: Unsigned,
    Sum<I, U1>: Unsigned,
{
    type Output = CQ<Sum<I, U1>, F>;

    /// Subtracts two `CQ<I, F>` values, producing `CQ<Sum<I, U1>, F>`.
    ///
    /// The extra integer bit guarantees no overflow.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The right-hand operand.
    ///
    /// # Returns
    ///
    /// The exact difference as `CQ<Sum<I, U1>, F>`.
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        CQ {
            re: self.re - rhs.re,
            im: self.im - rhs.im,
        }
    }
}

// ---------------------------------------------------------------------------
// CQ<I, F>: Neg
// ---------------------------------------------------------------------------

impl<I, F> ops::Neg for CQ<I, F>
where
    I: Unsigned + ops::Add<U1>,
    F: Unsigned,
    Sum<I, U1>: Unsigned,
{
    type Output = CQ<Sum<I, U1>, F>;

    /// Negates a `CQ<I, F>` value, producing `CQ<Sum<I, U1>, F>`.
    ///
    /// The extra integer bit handles the `MIN` case without wrapping.
    ///
    /// # Returns
    ///
    /// The exact negation as `CQ<Sum<I, U1>, F>`.
    #[inline]
    fn neg(self) -> Self::Output {
        CQ {
            re: -self.re,
            im: -self.im,
        }
    }
}

// ---------------------------------------------------------------------------
// CQ<I, F>: Mul (complex)
// ---------------------------------------------------------------------------

impl<I, F> ops::Mul for CQ<I, F>
where
    I: Unsigned + ops::Add<I>,
    F: Unsigned + ops::Add<F>,
    Sum<I, I>: Unsigned + ops::Add<U1>,
    Sum<Sum<I, I>, U1>: Unsigned,
    Sum<F, F>: Unsigned,
{
    type Output = CQ<Sum<Sum<I, I>, U1>, Sum<F, F>>;

    /// Multiplies two `CQ<I, F>` values: `(ac - bd) + (ad + bc)i`.
    ///
    /// Produces `CQ<Sum<Sum<I, I>, U1>, Sum<F, F>>` — one integer bit wider
    /// than a real multiply, because each component is a sum or difference of
    /// two full products. The result cannot overflow; use `.truncate()` or
    /// `.saturate()` to narrow it.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The right-hand operand.
    ///
    /// # Returns
    ///
    /// The exact product as `CQ<Sum<Sum<I, I>, U1>, Sum<F, F>>`.
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        let re = self.re.raw() as i128 * rhs.re.raw() as i128
            - self.im.raw() as i128 * rhs.im.raw() as i128;
        let im = self.re.raw() as i128 * rhs.im.raw() as i128
            + self.im.raw() as i128 * rhs.re.raw() as i128;
        CQ {
            re: Q::<Sum<Sum<I, I>, U1>, Sum<F, F>>::from_raw(re as i64),
            im: Q::<Sum<Sum<I, I>, U1>, Sum<F, F>>::from_raw(im as i64),
        }
    }
}

// ---------------------------------------------------------------------------
// CQ<I, F>: scalar Mul by Q (CQ * Q and Q * CQ)
// ---------------------------------------------------------------------------

impl<I, F> ops::Mul<Q<I, F>> for CQ<I, F>
where
    I: Unsigned + ops::Add<I>,
    F: Unsigned + ops::Add<F>,
    Sum<I, I>: Unsigned,
    Sum<F, F>: Unsigned,
{
    type Output = CQ<Sum<I, I>, Sum<F, F>>;

    /// Scales a complex value by a real `Q<I, F>`, producing
    /// `CQ<Sum<I, I>, Sum<F, F>>`.
    ///
    /// Unlike complex × complex, no extra integer bit is needed: each
    /// component is a single full product.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The real scalar.
    ///
    /// # Returns
    ///
    /// The scaled value as `CQ<Sum<I, I>, Sum<F, F>>`.
    #[inline]
    fn mul(self, rhs: Q<I, F>) -> Self::Output {
        CQ {
            re: self.re * rhs,
            im: self.im * rhs,
        }
    }
}

impl<I, F> ops::Mul<CQ<I, F>> for Q<I, F>
where
    I: Unsigned + ops::Add<I>,
    F: Unsigned + ops::Add<F>,
    Sum<I, I>: Unsigned,
    Sum<F, F>: Unsigned,
{
    type Output = CQ<Sum<I, I>, Sum<F, F>>;

    /// Scales a complex value by a real `Q<I, F>` from the left, producing
    /// `CQ<Sum<I, I>, Sum<F, F>>`.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The complex operand.
    ///
    /// # Returns
    ///
    /// The scaled value as `CQ<Sum<I, I>, Sum<F, F>>`.
    #[inline]
    fn mul(self, rhs: CQ<I, F>) -> Self::Output {
        CQ {
            re: self * rhs.re,
            im: self * rhs.im,
        }
    }
}

// ---------------------------------------------------------------------------
// CQ<I, F>: Shl, Shr (componentwise)
// ---------------------------------------------------------------------------

impl<I: Unsigned, F: Unsigned> ops::Shl<u32> for CQ<I, F> {
    type Output = Self;

    /// Shifts both components left by `shift` bits.
    ///
    /// # Arguments
    ///
    /// * `shift` - Number of bits to shift left.
    ///
    /// # Returns
    ///
    /// The shifted value, each component masked to the bit width.
    #[inline]
    fn shl(self, shift: u32) -> Self {
        Self {
            re: self.re << shift,
            im: self.im << shift,
        }
    }
}

impl<I: Unsigned, F: Unsigned> ops::ShlAssign<u32> for CQ<I, F> {
    /// Shifts both components left in place.
    ///
    /// # Arguments
    ///
    /// * `shift` - Number of bits to shift left.
    #[inline]
    fn shl_assign(&mut self, shift: u32) {
        *self = *self << shift;
    }
}

impl<I: Unsigned, F: Unsigned> ops::Shr<u32> for CQ<I, F> {
    type Output = Self;

    /// Arithmetic right shift (sign-preserving) of both components.
    ///
    /// # Arguments
    ///
    /// * `shift` - Number of bits to shift right.
    ///
    /// # Returns
    ///
    /// The shifted value with sign extension on each component.
    #[inline]
    fn shr(self, shift: u32) -> Self {
        Self {
            re: self.re >> shift,
            im: self.im >> shift,
        }
    }
}

impl<I: Unsigned, F: Unsigned> ops::ShrAssign<u32> for CQ<I, F> {
    /// Shifts both components right in place (arithmetic / sign-preserving).
    ///
    /// # Arguments
    ///
    /// * `shift` - Number of bits to shift right.
    #[inline]
    fn shr_assign(&mut self, shift: u32) {
        *self = *self >> shift;
    }
}

// ---------------------------------------------------------------------------
// CQ<I, F>: Sum (componentwise, logarithmic bit growth)
// ---------------------------------------------------------------------------

impl<I, F, IO, FO> core::iter::Sum<CQ<I, F>> for CQ<IO, FO>
where
    I: Unsigned,
    F: Unsigned,
    IO: Unsigned,
    FO: Unsigned,
{
    /// Sums an iterator of `CQ<I, F>` into a caller-chosen accumulator
    /// `CQ<IO, FO>`, componentwise.
    ///
    /// Each component grows like the scalar [`Q`] sum: summing `N` values needs
    /// only `ceil(log2(N))` extra integer bits over the input. Real and
    /// imaginary partial sums accumulate in 128 bits, so no intermediate
    /// overflow occurs; choose `IO` at least `I + ceil(log2(N))` or each
    /// component wraps to `IO + FO` bits. An empty iterator yields
    /// [`CQ::ZERO`].
    #[inline]
    fn sum<It: Iterator<Item = CQ<I, F>>>(iter: It) -> Self {
        let mut re_acc: i128 = 0;
        let mut im_acc: i128 = 0;
        for c in iter {
            re_acc += c.re.raw() as i128;
            im_acc += c.im.raw() as i128;
        }
        let shift = FO::U32 as i32 - F::U32 as i32;
        let (re, im) = if shift >= 0 {
            (re_acc << shift, im_acc << shift)
        } else {
            (re_acc >> (-shift), im_acc >> (-shift))
        };
        CQ {
            re: Q::<IO, FO>::from_raw(re as i64),
            im: Q::<IO, FO>::from_raw(im as i64),
        }
    }
}

impl<'a, I, F, IO, FO> core::iter::Sum<&'a CQ<I, F>> for CQ<IO, FO>
where
    I: Unsigned,
    F: Unsigned,
    IO: Unsigned,
    FO: Unsigned,
{
    /// Sums an iterator of `&CQ<I, F>` into a caller-chosen accumulator
    /// `CQ<IO, FO>`; see the owned `Sum` impl for the bit-growth contract.
    #[inline]
    fn sum<It: Iterator<Item = &'a CQ<I, F>>>(iter: It) -> Self {
        iter.copied().sum()
    }
}

impl<IO: Unsigned, FO: Unsigned> CQ<IO, FO> {
    /// Checked sum: accumulates an iterator of `CQ<I, F>` into `CQ<IO, FO>`,
    /// componentwise, erroring if either component's total does not fit.
    ///
    /// The checked counterpart to the [`Sum`](core::iter::Sum) impl: identical
    /// 128-bit per-component accumulation, but returns
    /// [`FixedError::OutOfRange`] instead of wrapping when either the real or
    /// imaginary total falls outside `[MIN, MAX]`. An empty iterator yields
    /// `Ok(`[`ZERO`](CQ::ZERO)`)`.
    ///
    /// # Errors
    ///
    /// Returns [`FixedError::OutOfRange`] if either component does not fit
    /// `Q<IO, FO>`.
    #[inline]
    pub fn try_sum<I, F, It>(iter: It) -> Result<Self, FixedError>
    where
        I: Unsigned,
        F: Unsigned,
        It: IntoIterator<Item = CQ<I, F>>,
    {
        let mut re_acc: i128 = 0;
        let mut im_acc: i128 = 0;
        for c in iter {
            re_acc += c.re.raw() as i128;
            im_acc += c.im.raw() as i128;
        }
        let shift = FO::U32 as i32 - F::U32 as i32;
        let (re, im) = if shift >= 0 {
            (re_acc << shift, im_acc << shift)
        } else {
            (re_acc >> (-shift), im_acc >> (-shift))
        };
        let lo = Q::<IO, FO>::MIN.raw() as i128;
        let hi = Q::<IO, FO>::MAX.raw() as i128;
        if re < lo || re > hi || im < lo || im > hi {
            return Err(FixedError::OutOfRange);
        }
        Ok(CQ {
            re: Q::<IO, FO>::from_raw(re as i64),
            im: Q::<IO, FO>::from_raw(im as i64),
        })
    }
}
