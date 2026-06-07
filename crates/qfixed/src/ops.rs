//! `core::ops` trait implementations for `Q` and `UQ`.
//!
//! Implements `Add`, `Sub`, `Neg` (Q only), `Mul`, `Shl`, `Shr`, `BitAnd`,
//! `BitOr`.
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
