//! Error type for the fallible (`try_*`) conversions.

use core::fmt;

/// Error returned by the `try_*` conversions when a value cannot be
/// represented in the target fixed-point type.
///
/// The infallible constructors (`from_count`, `from_bits`, `from_f64`) never
/// produce this — they apply defined wrapping/saturating behavior instead, in
/// the spirit of a SystemC-style overflow mode. Use the `try_*` variants when
/// a testbench needs to detect or assert on out-of-range values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FixedError {
    /// The value lies outside the representable range of the target type, or
    /// (for `try_from_bits`) sets bits above the low `I + F`.
    OutOfRange,

    /// The value's magnitude fits, but it is not exactly representable in the
    /// target type — fractional precision would be lost (returned by
    /// `try_narrow`).
    Inexact,

    /// The floating-point value was NaN or infinite.
    NotFinite,
}

impl fmt::Display for FixedError {
    /// Writes a human-readable description of the error.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter to write to.
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating success or formatting error.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::OutOfRange => "value out of range for the target fixed-point type",
            Self::Inexact => "value not exactly representable in the target fixed-point type",
            Self::NotFinite => "floating-point value was NaN or infinite",
        };
        f.write_str(msg)
    }
}

impl core::error::Error for FixedError {}
