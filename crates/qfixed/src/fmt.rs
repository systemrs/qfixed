//! Formatting for `Q` and `UQ`.
//!
//! Rust's `core::fmt` dispatches each format specifier to a fixed set of
//! traits — there is no way to register a custom letter such as `{:D}`. So the
//! fixed-point views are mapped onto the standard traits that already match
//! their meaning:
//!
//! - `{}` ([`Display`](fmt::Display)) — the real-world decimal value, honoring
//!   precision/width/sign/fill (delegated to `f64`).
//! - `{:?}` ([`Debug`](fmt::Debug)) — a self-describing line with the Q tag,
//!   the raw register word in hex, and the value, e.g. `Q12.4(0x0108 = 16.5)`.
//! - `{:x}`/`{:X}`/`{:b}`/`{:o}` ([`LowerHex`](fmt::LowerHex) etc.) — the raw
//!   two's-complement **bit pattern** of the backing register, zero-padded to
//!   the type width. This is the bit-accurate view an RTL engineer expects (it
//!   always agrees with [`to_bits`](crate::Q::to_bits)); `#` adds the `0x`/
//!   `0b`/`0o` prefix.
//! - `{:e}`/`{:E}` ([`LowerExp`](fmt::LowerExp) etc.) — scientific decimal of
//!   the value (delegated to `f64`).
//!
//! Value-scaled hex (`0x10.8`) and a decibel view are intentionally left to
//! later, named adapter methods rather than overloading these letters.

use core::fmt::{self, Write};

use typenum::Unsigned;

use crate::q::Q;
use crate::uq::UQ;

/// Which integer base [`fmt_radix`] should render.
#[derive(Clone, Copy)]
enum Radix {
    LowerHex,
    UpperHex,
    Binary,
    Octal,
}

/// Fixed-capacity stack buffer for an integer's rendered digits.
///
/// The digits are built here so they can be handed to
/// `Formatter::pad_integral`, which then applies the caller's width, the `#`
/// prefix, and sign-aware zero padding. 64 bytes is the widest output we can
/// produce (a 64-bit value in binary), so writes never overflow it.
struct DigitBuf {
    bytes: [u8; 64],
    len: usize,
}

impl DigitBuf {
    const fn new() -> Self {
        Self {
            bytes: [0; 64],
            len: 0,
        }
    }

    /// The written prefix as a string. Only ASCII digits from integer
    /// formatting are ever stored, so this is always valid UTF-8.
    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.bytes[..self.len]).unwrap_or("")
    }
}

impl fmt::Write for DigitBuf {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let end = self.len + s.len();
        let dst = self.bytes.get_mut(self.len..end).ok_or(fmt::Error)?;
        dst.copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}

/// Renders the low `total_bits` of `bits` in the given base, zero-padded to the
/// type's natural width, then applies the formatter's flags via
/// `pad_integral`.
///
/// `bits` is treated as an unsigned two's-complement pattern (negatives print
/// as e.g. `fef8`, never `-108`), so `is_nonnegative` is always `true`.
fn fmt_radix(bits: u64, total_bits: u32, f: &mut fmt::Formatter<'_>, radix: Radix) -> fmt::Result {
    let mut buf = DigitBuf::new();
    let prefix = match radix {
        Radix::LowerHex => {
            let width = total_bits.div_ceil(4) as usize;
            write!(buf, "{bits:0>width$x}")?;
            "0x"
        }
        Radix::UpperHex => {
            let width = total_bits.div_ceil(4) as usize;
            write!(buf, "{bits:0>width$X}")?;
            "0x"
        }
        Radix::Binary => {
            let width = total_bits as usize;
            write!(buf, "{bits:0>width$b}")?;
            "0b"
        }
        Radix::Octal => {
            let width = total_bits.div_ceil(3) as usize;
            write!(buf, "{bits:0>width$o}")?;
            "0o"
        }
    };
    f.pad_integral(true, prefix, buf.as_str())
}

impl<I: Unsigned, F: Unsigned> fmt::Display for Q<I, F> {
    /// Displays the real-world decimal value.
    ///
    /// Precision, width, sign, and fill flags are honored by delegating to
    /// `f64`'s `Display`. With no explicit precision, prints the shortest
    /// decimal that round-trips the value (e.g. `16.5`, not `16.5000`).
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter to write to.
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating success or formatting error.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.to_f64(), f)
    }
}

impl<I: Unsigned, F: Unsigned> fmt::Debug for Q<I, F> {
    /// Formats as `Q12.4(0x0108 = 16.5)`: the Q tag, the raw two's-complement
    /// register word in hex (zero-padded to the type width), and the value.
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
            "Q{i}.{frac}(0x{bits:0>w$x} = {val})",
            w = (i + frac).div_ceil(4) as usize
        )
    }
}

impl<I: Unsigned, F: Unsigned> fmt::LowerHex for Q<I, F> {
    /// The raw bit pattern in lowercase hex, zero-padded to the type width;
    /// `#` adds the `0x` prefix.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_radix(self.to_bits() as u64, Self::TOTAL_BITS, f, Radix::LowerHex)
    }
}

impl<I: Unsigned, F: Unsigned> fmt::UpperHex for Q<I, F> {
    /// The raw bit pattern in uppercase hex, zero-padded to the type width;
    /// `#` adds the `0x` prefix.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_radix(self.to_bits() as u64, Self::TOTAL_BITS, f, Radix::UpperHex)
    }
}

impl<I: Unsigned, F: Unsigned> fmt::Binary for Q<I, F> {
    /// The raw bit pattern in binary, zero-padded to the full type width;
    /// `#` adds the `0b` prefix.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_radix(self.to_bits() as u64, Self::TOTAL_BITS, f, Radix::Binary)
    }
}

impl<I: Unsigned, F: Unsigned> fmt::Octal for Q<I, F> {
    /// The raw bit pattern in octal, zero-padded to the type width;
    /// `#` adds the `0o` prefix.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_radix(self.to_bits() as u64, Self::TOTAL_BITS, f, Radix::Octal)
    }
}

impl<I: Unsigned, F: Unsigned> fmt::LowerExp for Q<I, F> {
    /// Scientific decimal of the value (delegated to `f64`); honors precision.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerExp::fmt(&self.to_f64(), f)
    }
}

impl<I: Unsigned, F: Unsigned> fmt::UpperExp for Q<I, F> {
    /// Scientific decimal of the value (delegated to `f64`); honors precision.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperExp::fmt(&self.to_f64(), f)
    }
}

impl<I: Unsigned, F: Unsigned> fmt::Display for UQ<I, F> {
    /// Displays the real-world decimal value.
    ///
    /// Precision, width, sign, and fill flags are honored by delegating to
    /// `f64`'s `Display`. With no explicit precision, prints the shortest
    /// decimal that round-trips the value (e.g. `0.5`, not `0.5000`).
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter to write to.
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating success or formatting error.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.to_f64(), f)
    }
}

impl<I: Unsigned, F: Unsigned> fmt::Debug for UQ<I, F> {
    /// Formats as `UQ1.7(0x40 = 0.5)`: the UQ tag, the raw register word in hex
    /// (zero-padded to the type width), and the value.
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
            "UQ{i}.{frac}(0x{bits:0>w$x} = {val})",
            w = (i + frac).div_ceil(4) as usize
        )
    }
}

impl<I: Unsigned, F: Unsigned> fmt::LowerHex for UQ<I, F> {
    /// The raw bit pattern in lowercase hex, zero-padded to the type width;
    /// `#` adds the `0x` prefix.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_radix(self.to_bits(), Self::TOTAL_BITS, f, Radix::LowerHex)
    }
}

impl<I: Unsigned, F: Unsigned> fmt::UpperHex for UQ<I, F> {
    /// The raw bit pattern in uppercase hex, zero-padded to the type width;
    /// `#` adds the `0x` prefix.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_radix(self.to_bits(), Self::TOTAL_BITS, f, Radix::UpperHex)
    }
}

impl<I: Unsigned, F: Unsigned> fmt::Binary for UQ<I, F> {
    /// The raw bit pattern in binary, zero-padded to the full type width;
    /// `#` adds the `0b` prefix.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_radix(self.to_bits(), Self::TOTAL_BITS, f, Radix::Binary)
    }
}

impl<I: Unsigned, F: Unsigned> fmt::Octal for UQ<I, F> {
    /// The raw bit pattern in octal, zero-padded to the type width;
    /// `#` adds the `0o` prefix.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_radix(self.to_bits(), Self::TOTAL_BITS, f, Radix::Octal)
    }
}

impl<I: Unsigned, F: Unsigned> fmt::LowerExp for UQ<I, F> {
    /// Scientific decimal of the value (delegated to `f64`); honors precision.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerExp::fmt(&self.to_f64(), f)
    }
}

impl<I: Unsigned, F: Unsigned> fmt::UpperExp for UQ<I, F> {
    /// Scientific decimal of the value (delegated to `f64`); honors precision.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperExp::fmt(&self.to_f64(), f)
    }
}
