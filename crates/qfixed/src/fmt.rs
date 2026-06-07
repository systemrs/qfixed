//! `Display` and `Debug` formatting for `Q` and `UQ`.

use core::fmt;

use typenum::Unsigned;

use crate::q::Q;
use crate::uq::UQ;

impl<I: Unsigned, F: Unsigned> fmt::Display for Q<I, F> {
    /// Displays the decimal value with 4 fractional digits.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter to write to.
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating success or formatting error.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = self.to_f64();
        write!(f, "{val:.4}")
    }
}

impl<I: Unsigned, F: Unsigned> fmt::Debug for Q<I, F> {
    /// Formats as `Q12.4(0x0108 = 16.5000)`.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter to write to.
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating success or formatting error.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let i = I::U32;
        let frac = F::U32;
        let bits = self.to_bits();
        let val = self.to_f64();
        write!(
            f,
            "Q{i}.{frac}(0x{bits:0>w$x} = {val:.4})",
            w = (i + frac).div_ceil(4) as usize
        )
    }
}

impl<I: Unsigned, F: Unsigned> fmt::Display for UQ<I, F> {
    /// Displays the decimal value with 4 fractional digits.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter to write to.
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating success or formatting error.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = self.to_f64();
        write!(f, "{val:.4}")
    }
}

impl<I: Unsigned, F: Unsigned> fmt::Debug for UQ<I, F> {
    /// Formats as `UQ1.7(0x40 = 0.5000)`.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter to write to.
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating success or formatting error.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let i = I::U32;
        let frac = F::U32;
        let bits = self.to_bits();
        let val = self.to_f64();
        write!(
            f,
            "UQ{i}.{frac}(0x{bits:0>w$x} = {val:.4})",
            w = (i + frac).div_ceil(4) as usize
        )
    }
}
