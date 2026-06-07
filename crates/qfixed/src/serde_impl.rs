//! Serde support for `Q`, `UQ`, and `CQ` (behind the `serde` feature).
//!
//! Serializes as the raw bit pattern: `u64` for both `Q` and `UQ`, and a
//! `(u64, u64)` tuple of real/imaginary raw bits for `CQ`.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use typenum::Unsigned;

use crate::cq::CQ;
use crate::q::Q;
use crate::uq::UQ;

impl<I: Unsigned, F: Unsigned> Serialize for Q<I, F> {
    /// Serializes the raw bit pattern as `u64`.
    ///
    /// # Arguments
    ///
    /// * `serializer` - The serde serializer.
    ///
    /// # Returns
    ///
    /// The serialization result.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_bits().serialize(serializer)
    }
}

impl<'de, I: Unsigned, F: Unsigned> Deserialize<'de> for Q<I, F> {
    /// Deserializes from `u64` raw bits.
    ///
    /// # Arguments
    ///
    /// * `deserializer` - The serde deserializer.
    ///
    /// # Returns
    ///
    /// The deserialized `Q<I, F>` value.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bits = u64::deserialize(deserializer)?;
        Ok(Self::from_bits(bits))
    }
}

impl<I: Unsigned, F: Unsigned> Serialize for UQ<I, F> {
    /// Serializes the raw bit representation as `u64`.
    ///
    /// # Arguments
    ///
    /// * `serializer` - The serde serializer.
    ///
    /// # Returns
    ///
    /// The serialization result.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_bits().serialize(serializer)
    }
}

impl<'de, I: Unsigned, F: Unsigned> Deserialize<'de> for UQ<I, F> {
    /// Deserializes from `u64` raw bits.
    ///
    /// # Arguments
    ///
    /// * `deserializer` - The serde deserializer.
    ///
    /// # Returns
    ///
    /// The deserialized `UQ<I, F>` value.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bits = u64::deserialize(deserializer)?;
        Ok(Self::from_bits(bits))
    }
}

impl<I: Unsigned, F: Unsigned> Serialize for CQ<I, F> {
    /// Serializes as a `(u64, u64)` tuple of the real and imaginary raw bits.
    ///
    /// # Arguments
    ///
    /// * `serializer` - The serde serializer.
    ///
    /// # Returns
    ///
    /// The serialization result.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_bits().serialize(serializer)
    }
}

impl<'de, I: Unsigned, F: Unsigned> Deserialize<'de> for CQ<I, F> {
    /// Deserializes from a `(u64, u64)` tuple of real and imaginary raw bits.
    ///
    /// # Arguments
    ///
    /// * `deserializer` - The serde deserializer.
    ///
    /// # Returns
    ///
    /// The deserialized `CQ<I, F>` value.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let (re, im) = <(u64, u64)>::deserialize(deserializer)?;
        Ok(Self::from_bits(re, im))
    }
}
