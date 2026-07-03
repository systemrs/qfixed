//! Memory-compact containers that store fixed-point values in the smallest
//! integer that fits their bit width.
//!
//! A scalar [`Q`]/[`UQ`] is always one `i64`/`u64` and a [`CQ`] is two,
//! regardless of `I + F`. That is convenient for
//! arithmetic (every intermediate widens into a 128-bit accumulator) but
//! wasteful for large buffers: a `[Q<U4, U4>; N]` spends 8 bytes per 8-bit
//! sample. These containers keep the scalar types — and their whole `const fn`
//! API — untouched, and shrink storage only where the bytes actually are: in
//! bulk arrays.
//!
//! Two packing strategies are offered:
//!
//! - **Byte-granular** ([`Packable`]): each element is stored in the smallest
//!   primitive integer that holds `I + F` bits (`i8`/`i16`/`i32`/`i64`, and the
//!   unsigned analogues for `UQ`). A `CQ` element is an interleaved `[repr; 2]`
//!   real/imaginary pair. Fast to read and write; used by [`PackedArray`] and
//!   [`PackedVec`]. Up to 8× smaller than `i64` for ≤ 8-bit types.
//! - **Bit-exact** ([`BitPackable`]): each element occupies *exactly* `I + F`
//!   bits (a `CQ` element occupies `2 * (I + F)`), packed across byte
//!   boundaries with no waste — e.g. 12-bit ADC samples pack at 12 bits each,
//!   not 16. Used by [`PackedBits`] and [`PackedBitsVec`]. Denser but slower;
//!   access does a read-modify-write shift/mask and does not vectorize.
//!
//! Packing is lossless: values already fit `I + F` bits by construction, so a
//! `get` reconstructs the exact original. Elements are returned **by value**
//! (packed storage cannot hand out a `&Q`), so mutation is `get` → compute →
//! `set` rather than in-place.
//!
//! ```
//! use qfixed::Q;
//! use qfixed::PackedArray;
//! use qfixed::typenum::{U4, U12};
//!
//! // 16 samples of Q12.4 (16-bit) in an i16-backed array: 32 bytes, not 128.
//! let mut buf = PackedArray::<Q<U12, U4>, 16>::new();
//! buf.set(0, Q::from(3i32));
//! buf.set(1, Q::from(-5i32));
//! assert_eq!(buf.get(0).unwrap().to_f64(), 3.0);
//! assert_eq!(core::mem::size_of_val(&buf), 32);
//! ```

use core::marker::PhantomData;
use core::ops::Add;

use typenum::{Sum, Unsigned};

use crate::cq::CQ;
use crate::q::Q;
use crate::uq::UQ;

/// Seals [`Repr`], [`Packable`], and [`BitPackable`] against downstream
/// implementations, so the crate controls the backing-integer mapping.
mod sealed {
    /// Marker implemented only for the types this crate blesses.
    pub trait Sealed {}
}

// ===========================================================================
// Repr: type-level bit width -> smallest primitive integer
// ===========================================================================

/// Maps a type-level total bit width (`I + F`, as a [`typenum`] unsigned
/// integer) to the smallest primitive integers that can hold it.
///
/// Implemented for `U1..=U64`: `1..=8` bits use `i8`/`u8`, `9..=16` use
/// `i16`/`u16`, `17..=32` use `i32`/`u32`, and `33..=64` use `i64`/`u64`. This
/// is the backing-selection engine behind [`Packable`]. It is a sealed,
/// implementation-detail trait: it is `pub` only because it appears in
/// `Packable`'s public interface, but the enclosing module is private and it is
/// never re-exported, so downstream crates can neither name nor implement it.
pub trait Repr: sealed::Sealed {
    /// Smallest signed primitive that holds the width.
    type Signed: Copy + Default;

    /// Smallest unsigned primitive that holds the width.
    type Unsigned: Copy + Default;

    /// Packs the low bits of a two's-complement pattern into the signed repr.
    ///
    /// # Arguments
    ///
    /// * `bits` - The bit pattern; only the low width bits are meaningful.
    ///
    /// # Returns
    ///
    /// The bits reinterpreted in the signed backing type.
    fn pack_signed(bits: u64) -> Self::Signed;

    /// Widens the signed repr back to the low bits of a `u64` pattern.
    ///
    /// # Arguments
    ///
    /// * `value` - The stored signed backing value.
    ///
    /// # Returns
    ///
    /// The low width bits as a `u64` (zero-extended; the caller masks).
    fn unpack_signed(value: Self::Signed) -> u64;

    /// Packs the low bits of a pattern into the unsigned repr.
    ///
    /// # Arguments
    ///
    /// * `bits` - The bit pattern; only the low width bits are meaningful.
    ///
    /// # Returns
    ///
    /// The bits in the unsigned backing type.
    fn pack_unsigned(bits: u64) -> Self::Unsigned;

    /// Widens the unsigned repr back to a `u64` pattern.
    ///
    /// # Arguments
    ///
    /// * `value` - The stored unsigned backing value.
    ///
    /// # Returns
    ///
    /// The value as a `u64`.
    fn unpack_unsigned(value: Self::Unsigned) -> u64;
}

macro_rules! impl_repr {
    ($signed:ty, $unsigned:ty; $($width:ident),+ $(,)?) => {
        $(
            impl sealed::Sealed for typenum::$width {}

            impl Repr for typenum::$width {
                type Signed = $signed;
                type Unsigned = $unsigned;

                #[inline]
                fn pack_signed(bits: u64) -> Self::Signed {
                    bits as $unsigned as $signed
                }

                #[inline]
                fn unpack_signed(value: Self::Signed) -> u64 {
                    value as $unsigned as u64
                }

                #[inline]
                fn pack_unsigned(bits: u64) -> Self::Unsigned {
                    bits as $unsigned
                }

                #[inline]
                fn unpack_unsigned(value: Self::Unsigned) -> u64 {
                    value as u64
                }
            }
        )+
    };
}

impl_repr!(i8, u8; U1, U2, U3, U4, U5, U6, U7, U8);
impl_repr!(i16, u16; U9, U10, U11, U12, U13, U14, U15, U16);
impl_repr!(
    i32, u32; U17, U18, U19, U20, U21, U22, U23, U24, U25, U26, U27, U28, U29,
    U30, U31, U32
);
impl_repr!(
    i64, u64; U33, U34, U35, U36, U37, U38, U39, U40, U41, U42, U43, U44, U45,
    U46, U47, U48, U49, U50, U51, U52, U53, U54, U55, U56, U57, U58, U59, U60,
    U61, U62, U63, U64
);

// ===========================================================================
// Packable: byte-granular smallest-primitive backing
// ===========================================================================

/// A fixed-point value that can be stored in the smallest primitive integer
/// holding its bit width, for use in [`PackedArray`] and [`PackedVec`].
///
/// Implemented for [`Q`], [`UQ`], and [`CQ`] whose total width `I + F` is
/// between 1 and 64 bits. The associated [`Backing`](Packable::Backing) is the
/// storage type: a single `i8`/`i16`/`i32`/`i64` (or unsigned analogue for
/// `UQ`), or an interleaved `[repr; 2]` for `CQ`. Sealed: downstream crates
/// cannot implement it.
pub trait Packable: Copy + sealed::Sealed {
    /// The smallest primitive (or interleaved pair) that stores one element.
    type Backing: Copy + Default;

    /// Encodes `self` into its packed backing representation.
    ///
    /// # Returns
    ///
    /// The backing value to store.
    fn to_backing(self) -> Self::Backing;

    /// Reconstructs a value from its packed backing representation.
    ///
    /// # Arguments
    ///
    /// * `backing` - The stored backing value.
    ///
    /// # Returns
    ///
    /// The exact original value.
    fn from_backing(backing: Self::Backing) -> Self;
}

impl<I, F> sealed::Sealed for Q<I, F> {}
impl<I, F> sealed::Sealed for UQ<I, F> {}
impl<I, F> sealed::Sealed for CQ<I, F> {}

impl<I, F> Packable for Q<I, F>
where
    I: Unsigned + Add<F>,
    F: Unsigned,
    Sum<I, F>: Repr,
{
    type Backing = <Sum<I, F> as Repr>::Signed;

    #[inline]
    fn to_backing(self) -> Self::Backing {
        <Sum<I, F> as Repr>::pack_signed(self.to_bits())
    }

    #[inline]
    fn from_backing(backing: Self::Backing) -> Self {
        Self::from_bits(<Sum<I, F> as Repr>::unpack_signed(backing))
    }
}

impl<I, F> Packable for UQ<I, F>
where
    I: Unsigned + Add<F>,
    F: Unsigned,
    Sum<I, F>: Repr,
{
    type Backing = <Sum<I, F> as Repr>::Unsigned;

    #[inline]
    fn to_backing(self) -> Self::Backing {
        <Sum<I, F> as Repr>::pack_unsigned(self.to_bits())
    }

    #[inline]
    fn from_backing(backing: Self::Backing) -> Self {
        Self::from_bits(<Sum<I, F> as Repr>::unpack_unsigned(backing))
    }
}

impl<I, F> Packable for CQ<I, F>
where
    I: Unsigned + Add<F>,
    F: Unsigned,
    Sum<I, F>: Repr,
{
    type Backing = [<Sum<I, F> as Repr>::Signed; 2];

    #[inline]
    fn to_backing(self) -> Self::Backing {
        let (re, im) = self.to_bits();
        [
            <Sum<I, F> as Repr>::pack_signed(re),
            <Sum<I, F> as Repr>::pack_signed(im),
        ]
    }

    #[inline]
    fn from_backing(backing: Self::Backing) -> Self {
        Self::from_bits(
            <Sum<I, F> as Repr>::unpack_signed(backing[0]),
            <Sum<I, F> as Repr>::unpack_signed(backing[1]),
        )
    }
}

// ===========================================================================
// BitPackable: exact I+F-bit backing
// ===========================================================================

/// A fixed-point value that can be packed to exactly its `I + F` bits, for use
/// in [`PackedBits`] and [`PackedBitsVec`].
///
/// Each element is described as `LANES` lanes of `LANE_BITS` bits: [`Q`]/[`UQ`]
/// are a single lane of `I + F` bits, and [`CQ`] is two lanes (real then
/// imaginary) of `I + F` bits each. Implemented only for widths `1 <= I + F <=
/// 64` (the same range the scalar types allow), so a zero- or over-width
/// element type is rejected at compile time rather than dividing by zero or
/// silently corrupting. Sealed: downstream crates cannot implement it.
pub trait BitPackable: Copy + sealed::Sealed {
    /// Number of bits in each lane (`I + F`).
    const LANE_BITS: u32;

    /// Number of lanes per element (1 for `Q`/`UQ`, 2 for `CQ`).
    const LANES: usize;

    /// Returns the bit pattern of lane `index`.
    ///
    /// # Arguments
    ///
    /// * `index` - The lane to read, in `0..LANES`.
    ///
    /// # Returns
    ///
    /// The low `LANE_BITS` bits of that lane.
    fn lane(self, index: usize) -> u64;

    /// Rebuilds a value from its lane bit patterns.
    ///
    /// # Arguments
    ///
    /// * `lanes` - The lane patterns, of length `LANES`.
    ///
    /// # Returns
    ///
    /// The reconstructed value.
    fn from_lanes(lanes: &[u64]) -> Self;
}

impl<I, F> BitPackable for Q<I, F>
where
    I: Unsigned + Add<F>,
    F: Unsigned,
    Sum<I, F>: Repr,
{
    const LANE_BITS: u32 = Self::TOTAL_BITS;
    const LANES: usize = 1;

    #[inline]
    fn lane(self, _index: usize) -> u64 {
        self.to_bits()
    }

    #[inline]
    fn from_lanes(lanes: &[u64]) -> Self {
        Self::from_bits(lanes[0])
    }
}

impl<I, F> BitPackable for UQ<I, F>
where
    I: Unsigned + Add<F>,
    F: Unsigned,
    Sum<I, F>: Repr,
{
    const LANE_BITS: u32 = Self::TOTAL_BITS;
    const LANES: usize = 1;

    #[inline]
    fn lane(self, _index: usize) -> u64 {
        self.to_bits()
    }

    #[inline]
    fn from_lanes(lanes: &[u64]) -> Self {
        Self::from_bits(lanes[0])
    }
}

impl<I, F> BitPackable for CQ<I, F>
where
    I: Unsigned + Add<F>,
    F: Unsigned,
    Sum<I, F>: Repr,
{
    const LANE_BITS: u32 = Self::TOTAL_BITS;
    const LANES: usize = 2;

    #[inline]
    fn lane(self, index: usize) -> u64 {
        let (re, im) = self.to_bits();
        if index == 0 { re } else { im }
    }

    #[inline]
    fn from_lanes(lanes: &[u64]) -> Self {
        Self::from_bits(lanes[0], lanes[1])
    }
}

// ===========================================================================
// Bit-stream helpers
// ===========================================================================

/// Mask covering the low `n` bits.
///
/// # Arguments
///
/// * `n` - The number of low bits to set (`1..=64`).
///
/// # Returns
///
/// A `u64` with the low `n` bits set and the rest clear.
#[inline]
fn low_mask(n: u32) -> u64 {
    if n >= 64 { u64::MAX } else { (1u64 << n) - 1 }
}

/// Loads the `span` bytes starting at `start` into a little-endian `u128`
/// window (the shared read side of [`read_bits`] and [`write_bits`]).
///
/// # Arguments
///
/// * `buf` - The backing byte buffer.
/// * `start` - The first byte index to load.
/// * `span` - The number of bytes to load (`<= 9`, so it fits a `u128`).
///
/// # Returns
///
/// The loaded bytes, byte `j` occupying bits `8*j..8*j+8`.
#[inline]
fn load_window(buf: &[u8], start: usize, span: usize) -> u128 {
    let mut acc: u128 = 0;
    for j in 0..span {
        acc |= (buf[start + j] as u128) << (8 * j);
    }
    acc
}

/// Reads `n` bits (`n <= 64`) starting at `bit_offset` from a byte buffer.
///
/// Bits are laid out little-endian within the stream: element/lane `k` occupies
/// the half-open bit range `[k*bits, k*bits + bits)`.
///
/// # Arguments
///
/// * `buf` - The backing byte buffer.
/// * `bit_offset` - The starting bit index.
/// * `n` - The number of bits to read (`1..=64`).
///
/// # Returns
///
/// The `n` extracted bits, right-aligned.
#[inline]
fn read_bits(buf: &[u8], bit_offset: usize, n: u32) -> u64 {
    let start = bit_offset / 8;
    let shift = (bit_offset % 8) as u32;
    let span = (shift + n).div_ceil(8) as usize;
    let acc = load_window(buf, start, span);
    ((acc >> shift) as u64) & low_mask(n)
}

/// Writes the low `n` bits (`n <= 64`) of `value` at `bit_offset`, leaving the
/// surrounding bits untouched (read-modify-write).
///
/// # Arguments
///
/// * `buf` - The backing byte buffer, large enough to hold the touched bytes.
/// * `bit_offset` - The starting bit index.
/// * `n` - The number of bits to write (`1..=64`).
/// * `value` - The value whose low `n` bits are stored.
#[inline]
fn write_bits(buf: &mut [u8], bit_offset: usize, n: u32, value: u64) {
    let start = bit_offset / 8;
    let shift = (bit_offset % 8) as u32;
    let span = (shift + n).div_ceil(8) as usize;
    let mask = low_mask(n) as u128;
    let mut acc = load_window(buf, start, span);
    acc &= !(mask << shift);
    acc |= ((value as u128) & mask) << shift;
    for j in 0..span {
        buf[start + j] = (acc >> (8 * j)) as u8;
    }
}

/// Number of bits one `T` element occupies in a bit-packed buffer.
///
/// # Returns
///
/// `LANE_BITS * LANES` — `I + F` for `Q`/`UQ`, `2 * (I + F)` for `CQ`.
#[inline]
fn elem_bits<T: BitPackable>() -> usize {
    T::LANE_BITS as usize * T::LANES
}

/// Reads element `index` from a bit-packed byte buffer.
///
/// # Arguments
///
/// * `buf` - The backing byte buffer.
/// * `index` - The element index (caller guarantees it is in bounds).
///
/// # Returns
///
/// The reconstructed element.
#[inline]
fn read_element<T: BitPackable>(buf: &[u8], index: usize) -> T {
    let lane_bits = T::LANE_BITS as usize;
    let base = index * elem_bits::<T>();
    let mut lanes = [0u64; 2];
    for (k, lane) in lanes.iter_mut().enumerate().take(T::LANES) {
        *lane = read_bits(buf, base + k * lane_bits, T::LANE_BITS);
    }
    T::from_lanes(&lanes[..T::LANES])
}

/// Writes `value` as element `index` into a bit-packed byte buffer.
///
/// # Arguments
///
/// * `buf` - The backing byte buffer, large enough to hold the element.
/// * `index` - The element index.
/// * `value` - The value to store.
#[inline]
fn write_element<T: BitPackable>(buf: &mut [u8], index: usize, value: T) {
    let lane_bits = T::LANE_BITS as usize;
    let base = index * elem_bits::<T>();
    for k in 0..T::LANES {
        write_bits(buf, base + k * lane_bits, T::LANE_BITS, value.lane(k));
    }
}

// ===========================================================================
// PackedArray: fixed-capacity, byte-granular, no_std
// ===========================================================================

/// A fixed-capacity array of `N` fixed-point values, each stored in the
/// smallest primitive integer that fits its bit width.
///
/// The in-memory size is `N * size_of::<T::Backing>()` — up to 8× smaller than
/// `[T; N]` for ≤ 8-bit types. Fully `no_std`; needs no allocator. Elements are
/// returned by value.
#[derive(Clone, Copy)]
pub struct PackedArray<T: Packable, const N: usize> {
    /// The packed backing storage, one slot per element.
    data: [T::Backing; N],
}

impl<T: Packable, const N: usize> PackedArray<T, N> {
    /// Creates an array with every element zeroed.
    ///
    /// # Returns
    ///
    /// A new zero-filled `PackedArray`.
    #[inline]
    pub fn new() -> Self {
        Self {
            data: [T::Backing::default(); N],
        }
    }

    /// Creates an array by calling `f` with each index `0..N`.
    ///
    /// # Arguments
    ///
    /// * `f` - Produces the element for each index.
    ///
    /// # Returns
    ///
    /// A new `PackedArray` populated from `f`.
    #[inline]
    pub fn from_fn<Fun: FnMut(usize) -> T>(mut f: Fun) -> Self {
        Self {
            data: core::array::from_fn(|i| f(i).to_backing()),
        }
    }

    /// Creates a packed array from a plain `[T; N]`.
    ///
    /// # Arguments
    ///
    /// * `values` - The source array.
    ///
    /// # Returns
    ///
    /// A new `PackedArray` holding the same values.
    #[inline]
    pub fn from_array(values: [T; N]) -> Self {
        // `T: Packable: Copy`, so indexing copies; this is `from_fn` over the
        // source array.
        Self::from_fn(|i| values[i])
    }

    /// The number of elements (always `N`).
    ///
    /// # Returns
    ///
    /// `N`.
    #[inline]
    #[allow(clippy::unused_self)]
    pub const fn len(&self) -> usize {
        N
    }

    /// Whether the array is empty (`N == 0`).
    ///
    /// # Returns
    ///
    /// `true` if `N == 0`.
    #[inline]
    #[allow(clippy::unused_self)]
    pub const fn is_empty(&self) -> bool {
        N == 0
    }

    /// Returns the element at `index`, or `None` if out of bounds.
    ///
    /// # Arguments
    ///
    /// * `index` - The element index.
    ///
    /// # Returns
    ///
    /// `Some(value)` if `index < N`, else `None`.
    #[inline]
    pub fn get(&self, index: usize) -> Option<T> {
        if index < N {
            Some(T::from_backing(self.data[index]))
        } else {
            None
        }
    }

    /// Stores `value` at `index`.
    ///
    /// # Arguments
    ///
    /// * `index` - The element index.
    /// * `value` - The value to store.
    ///
    /// # Panics
    ///
    /// Panics if `index >= N`.
    #[inline]
    pub fn set(&mut self, index: usize, value: T) {
        assert!(
            index < N,
            "PackedArray: index {index} out of bounds (len {N})"
        );
        self.data[index] = value.to_backing();
    }

    /// Iterates over the elements by value.
    ///
    /// # Returns
    ///
    /// An iterator yielding each element in order.
    #[inline]
    #[allow(clippy::iter_without_into_iter)]
    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        (0..N).map(move |i| T::from_backing(self.data[i]))
    }

    /// Unpacks into a plain `[T; N]`.
    ///
    /// # Returns
    ///
    /// A fresh array of the reconstructed values.
    #[inline]
    pub fn to_array(&self) -> [T; N] {
        core::array::from_fn(|i| T::from_backing(self.data[i]))
    }
}

impl<T: Packable, const N: usize> Default for PackedArray<T, N> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// PackedVec: growable, byte-granular, alloc
// ===========================================================================

/// A growable vector of fixed-point values, each stored in the smallest
/// primitive integer that fits its bit width.
///
/// The `Vec`-like counterpart to [`PackedArray`]: up to 8× smaller than
/// `Vec<T>` for ≤ 8-bit types. Requires the `alloc` feature. Elements are
/// returned by value.
///
/// ```
/// use qfixed::Q;
/// use qfixed::PackedVec;
/// use qfixed::typenum::{U4, U4 as F4};
///
/// let mut v = PackedVec::<Q<U4, F4>>::new();
/// v.push(Q::from(1i32));
/// v.push(Q::from(-2i32));
/// assert_eq!(v.len(), 2);
/// assert_eq!(v.get(1).unwrap().to_f64(), -2.0);
/// // Two Q4.4 samples in an i8-backed vector: 2 bytes of payload.
/// let total: Vec<Q<U4, F4>> = v.iter().collect();
/// assert_eq!(total.len(), 2);
/// ```
#[cfg(feature = "alloc")]
#[derive(Clone)]
pub struct PackedVec<T: Packable> {
    /// The packed backing storage, one slot per element.
    data: alloc::vec::Vec<T::Backing>,
}

#[cfg(feature = "alloc")]
impl<T: Packable> PackedVec<T> {
    /// Creates an empty vector.
    ///
    /// # Returns
    ///
    /// A new empty `PackedVec`.
    #[inline]
    pub const fn new() -> Self {
        Self {
            data: alloc::vec::Vec::new(),
        }
    }

    /// Creates an empty vector with room for `capacity` elements.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The number of elements to preallocate.
    ///
    /// # Returns
    ///
    /// A new `PackedVec` with the requested backing capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: alloc::vec::Vec::with_capacity(capacity),
        }
    }

    /// The number of elements.
    ///
    /// # Returns
    ///
    /// The element count.
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Whether the vector is empty.
    ///
    /// # Returns
    ///
    /// `true` if there are no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Appends `value` to the end.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to append.
    #[inline]
    pub fn push(&mut self, value: T) {
        self.data.push(value.to_backing());
    }

    /// Removes and returns the last element, or `None` if empty.
    ///
    /// # Returns
    ///
    /// `Some(value)` if non-empty, else `None`.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.data.pop().map(T::from_backing)
    }

    /// Returns the element at `index`, or `None` if out of bounds.
    ///
    /// # Arguments
    ///
    /// * `index` - The element index.
    ///
    /// # Returns
    ///
    /// `Some(value)` if in bounds, else `None`.
    #[inline]
    pub fn get(&self, index: usize) -> Option<T> {
        self.data.get(index).map(|&b| T::from_backing(b))
    }

    /// Stores `value` at `index`.
    ///
    /// # Arguments
    ///
    /// * `index` - The element index.
    /// * `value` - The value to store.
    ///
    /// # Panics
    ///
    /// Panics if `index >= len`.
    #[inline]
    pub fn set(&mut self, index: usize, value: T) {
        let len = self.data.len();
        assert!(
            index < len,
            "PackedVec: index {index} out of bounds (len {len})"
        );
        self.data[index] = value.to_backing();
    }

    /// Removes all elements, keeping the allocation.
    #[inline]
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Iterates over the elements by value.
    ///
    /// # Returns
    ///
    /// An iterator yielding each element in order.
    #[inline]
    #[allow(clippy::iter_without_into_iter)]
    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        self.data.iter().map(|&b| T::from_backing(b))
    }
}

#[cfg(feature = "alloc")]
impl<T: Packable> Default for PackedVec<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "alloc")]
impl<T: Packable> FromIterator<T> for PackedVec<T> {
    #[inline]
    fn from_iter<It: IntoIterator<Item = T>>(iter: It) -> Self {
        Self {
            data: iter.into_iter().map(Packable::to_backing).collect(),
        }
    }
}

#[cfg(feature = "alloc")]
impl<T: Packable> Extend<T> for PackedVec<T> {
    #[inline]
    fn extend<It: IntoIterator<Item = T>>(&mut self, iter: It) {
        self.data.extend(iter.into_iter().map(Packable::to_backing));
    }
}

// ===========================================================================
// PackedBits: fixed-capacity, bit-exact, no_std
// ===========================================================================

/// A fixed byte-budget buffer that packs values at *exactly* `I + F` bits each
/// (a `CQ` element at `2 * (I + F)` bits), crossing byte boundaries with no
/// waste.
///
/// The capacity is `BYTES * 8 / bits_per_element` and is fixed at compile time
/// by the byte budget, so this is fully `no_std`. Denser than [`PackedArray`]
/// (e.g. 12-bit samples pack at 12 bits, not 16) but access is a slower
/// read-modify-write shift/mask. Elements are returned by value.
///
/// ```
/// use qfixed::Q;
/// use qfixed::PackedBits;
/// use qfixed::typenum::{U0, U12};
///
/// // 12-bit ADC samples in a 30-byte budget: 30*8/12 = 20 samples.
/// let mut adc = PackedBits::<Q<U12, U0>, 30>::new();
/// assert_eq!(adc.capacity(), 20);
/// adc.push(Q::from(2047i32));
/// adc.push(Q::from(-2048i32));
/// assert_eq!(adc.get(0).unwrap().to_count(), 2047);
/// assert_eq!(adc.get(1).unwrap().to_count(), -2048);
/// ```
///
/// A zero-width element type is rejected at compile time (it would otherwise
/// divide by zero when computing capacity):
///
/// ```compile_fail
/// # use qfixed::Q;
/// # use qfixed::PackedBits;
/// # use qfixed::typenum::U0;
/// let _ = PackedBits::<Q<U0, U0>, 8>::new(); // 0-bit element: no BitPackable
/// ```
///
/// So is an over-width (`I + F > 64`) element type:
///
/// ```compile_fail
/// # use qfixed::Q;
/// # use qfixed::PackedBits;
/// # use qfixed::typenum::{U32, U33};
/// let _ = PackedBits::<Q<U33, U32>, 16>::new(); // 65-bit element: rejected
/// ```
#[derive(Clone, Copy)]
pub struct PackedBits<T: BitPackable, const BYTES: usize> {
    /// The bit-packed byte buffer.
    data: [u8; BYTES],

    /// The number of elements currently stored.
    len: usize,

    /// Element-type marker.
    _marker: PhantomData<T>,
}

impl<T: BitPackable, const BYTES: usize> PackedBits<T, BYTES> {
    /// Creates an empty buffer.
    ///
    /// # Returns
    ///
    /// A new zeroed, empty `PackedBits`.
    #[inline]
    pub const fn new() -> Self {
        Self {
            data: [0u8; BYTES],
            len: 0,
            _marker: PhantomData,
        }
    }

    /// The maximum number of elements that fit in the byte budget.
    ///
    /// # Returns
    ///
    /// `BYTES * 8 / bits_per_element`.
    #[inline]
    pub fn capacity(&self) -> usize {
        (BYTES * 8) / elem_bits::<T>()
    }

    /// The number of elements currently stored.
    ///
    /// # Returns
    ///
    /// The element count.
    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Whether the buffer holds no elements.
    ///
    /// # Returns
    ///
    /// `true` if empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Appends `value`.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to append.
    ///
    /// # Panics
    ///
    /// Panics if the buffer is at capacity.
    #[inline]
    pub fn push(&mut self, value: T) {
        let cap = self.capacity();
        assert!(self.len < cap, "PackedBits: capacity {cap} exceeded");
        write_element(&mut self.data, self.len, value);
        self.len += 1;
    }

    /// Appends `value`, returning it back if the buffer is full.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to append.
    ///
    /// # Returns
    ///
    /// `Ok(())` if stored, or `Err(value)` if the buffer is at capacity.
    ///
    /// # Errors
    ///
    /// Returns `Err(value)` when the buffer is full.
    #[inline]
    pub fn try_push(&mut self, value: T) -> Result<(), T> {
        if self.len < self.capacity() {
            write_element(&mut self.data, self.len, value);
            self.len += 1;
            Ok(())
        } else {
            Err(value)
        }
    }

    /// Removes and returns the last element, or `None` if empty.
    ///
    /// # Returns
    ///
    /// `Some(value)` if non-empty, else `None`.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let value = read_element(&self.data, self.len - 1);
        self.len -= 1;
        Some(value)
    }

    /// Returns the element at `index`, or `None` if out of bounds.
    ///
    /// # Arguments
    ///
    /// * `index` - The element index.
    ///
    /// # Returns
    ///
    /// `Some(value)` if `index < len`, else `None`.
    #[inline]
    pub fn get(&self, index: usize) -> Option<T> {
        if index < self.len {
            Some(read_element(&self.data, index))
        } else {
            None
        }
    }

    /// Stores `value` at `index`.
    ///
    /// # Arguments
    ///
    /// * `index` - The element index.
    /// * `value` - The value to store.
    ///
    /// # Panics
    ///
    /// Panics if `index >= len`.
    #[inline]
    pub fn set(&mut self, index: usize, value: T) {
        assert!(
            index < self.len,
            "PackedBits: index {index} out of bounds (len {})",
            self.len
        );
        write_element(&mut self.data, index, value);
    }

    /// Removes all elements (does not zero the buffer bytes).
    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Iterates over the elements by value.
    ///
    /// # Returns
    ///
    /// An iterator yielding each element in order.
    #[inline]
    #[allow(clippy::iter_without_into_iter)]
    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        (0..self.len).map(move |i| read_element(&self.data, i))
    }
}

impl<T: BitPackable, const BYTES: usize> Default for PackedBits<T, BYTES> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// PackedBitsVec: growable, bit-exact, alloc
// ===========================================================================

/// A growable buffer that packs values at *exactly* `I + F` bits each, the
/// `Vec`-like counterpart to [`PackedBits`].
///
/// Grows its byte buffer on demand to the minimum needed. Requires the `alloc`
/// feature. Denser than [`PackedVec`] but with slower shift/mask access.
/// Elements are returned by value.
#[cfg(feature = "alloc")]
#[derive(Clone)]
pub struct PackedBitsVec<T: BitPackable> {
    /// The bit-packed byte buffer.
    data: alloc::vec::Vec<u8>,

    /// The number of elements currently stored.
    len: usize,

    /// Element-type marker.
    _marker: PhantomData<T>,
}

#[cfg(feature = "alloc")]
impl<T: BitPackable> PackedBitsVec<T> {
    /// Creates an empty buffer.
    ///
    /// # Returns
    ///
    /// A new empty `PackedBitsVec`.
    #[inline]
    pub const fn new() -> Self {
        Self {
            data: alloc::vec::Vec::new(),
            len: 0,
            _marker: PhantomData,
        }
    }

    /// The number of elements currently stored.
    ///
    /// # Returns
    ///
    /// The element count.
    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Whether the buffer holds no elements.
    ///
    /// # Returns
    ///
    /// `true` if empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Appends `value`, growing the buffer as needed.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to append.
    #[inline]
    pub fn push(&mut self, value: T) {
        let needed = ((self.len + 1) * elem_bits::<T>()).div_ceil(8);
        if self.data.len() < needed {
            self.data.resize(needed, 0);
        }
        write_element(self.data.as_mut_slice(), self.len, value);
        self.len += 1;
    }

    /// Removes and returns the last element, or `None` if empty.
    ///
    /// # Returns
    ///
    /// `Some(value)` if non-empty, else `None`.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let value = read_element(self.data.as_slice(), self.len - 1);
        self.len -= 1;
        Some(value)
    }

    /// Returns the element at `index`, or `None` if out of bounds.
    ///
    /// # Arguments
    ///
    /// * `index` - The element index.
    ///
    /// # Returns
    ///
    /// `Some(value)` if `index < len`, else `None`.
    #[inline]
    pub fn get(&self, index: usize) -> Option<T> {
        if index < self.len {
            Some(read_element(self.data.as_slice(), index))
        } else {
            None
        }
    }

    /// Stores `value` at `index`.
    ///
    /// # Arguments
    ///
    /// * `index` - The element index.
    /// * `value` - The value to store.
    ///
    /// # Panics
    ///
    /// Panics if `index >= len`.
    #[inline]
    pub fn set(&mut self, index: usize, value: T) {
        assert!(
            index < self.len,
            "PackedBitsVec: index {index} out of bounds (len {})",
            self.len
        );
        write_element(self.data.as_mut_slice(), index, value);
    }

    /// Removes all elements, keeping the allocation.
    #[inline]
    pub fn clear(&mut self) {
        self.data.clear();
        self.len = 0;
    }

    /// Iterates over the elements by value.
    ///
    /// # Returns
    ///
    /// An iterator yielding each element in order.
    #[inline]
    #[allow(clippy::iter_without_into_iter)]
    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        (0..self.len).map(move |i| read_element(self.data.as_slice(), i))
    }
}

#[cfg(feature = "alloc")]
impl<T: BitPackable> Default for PackedBitsVec<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "alloc")]
impl<T: BitPackable> FromIterator<T> for PackedBitsVec<T> {
    #[inline]
    fn from_iter<It: IntoIterator<Item = T>>(iter: It) -> Self {
        let mut out = Self::new();
        for value in iter {
            out.push(value);
        }
        out
    }
}

#[cfg(feature = "alloc")]
impl<T: BitPackable> Extend<T> for PackedBitsVec<T> {
    #[inline]
    fn extend<It: IntoIterator<Item = T>>(&mut self, iter: It) {
        for value in iter {
            self.push(value);
        }
    }
}

// ===========================================================================
// Unit tests for the low-level bit engine
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::{low_mask, read_bits, write_bits};

    /// Verifies write-then-read returns the masked value for every bit offset
    /// (spanning all intra-byte shifts) and every width `1..=64`, including the
    /// all-ones and sign-bit-set patterns.
    #[test]
    fn read_after_write_all_shifts_and_widths() {
        for bit_offset in 0..24usize {
            for n in 1u32..=64 {
                for &raw in &[
                    0u64,
                    u64::MAX,
                    1,
                    0x8000_0000_0000_0000,
                    0x0F0F_0F0F_0F0F_0F0F,
                ] {
                    let mut buf = [0xA5u8; 32];
                    write_bits(&mut buf, bit_offset, n, raw);
                    assert_eq!(
                        read_bits(&buf, bit_offset, n),
                        raw & low_mask(n),
                        "offset {bit_offset} n {n} raw {raw:#x}",
                    );
                }
            }
        }
    }

    /// Verifies a write disturbs only its own bit range — the bits before and
    /// after stay intact (the read-modify-write mask is correct).
    #[test]
    fn write_preserves_neighbours() {
        for bit_offset in 0..16usize {
            for n in 1u32..=48 {
                let mut buf = [0xFFu8; 32];
                write_bits(&mut buf, bit_offset, n, 0);
                if bit_offset > 0 {
                    assert_eq!(
                        read_bits(&buf, 0, bit_offset as u32),
                        low_mask(bit_offset as u32),
                        "leading bits clobbered at offset {bit_offset} n {n}",
                    );
                }
                assert_eq!(
                    read_bits(&buf, bit_offset + n as usize, 8),
                    0xFF,
                    "trailing bits clobbered at offset {bit_offset} n {n}",
                );
                assert_eq!(read_bits(&buf, bit_offset, n), 0);
            }
        }
    }
}
