//! Quantity type representing lots as u128 values.
//!
//! Quantity is a strongly-typed newtype around u128 that represents
//! economic quantities in lots. This ensures type safety and prevents
//! accidental mixing of quantity values with other integer types.

use crate::instrument_spec::{quantity_from_decimal_string, InstrumentSpec};
use crate::QuantityParseError;

/// Quantity in lots (u128).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Quantity(u128);

impl Quantity {
    /// Create a new Quantity from lots.
    pub fn new(lots: u128) -> Self {
        Quantity(lots)
    }

    /// Get the inner lots value.
    pub fn inner(&self) -> u128 {
        self.0
    }

    /// Create a new Quantity from a u128 value.
    ///
    /// This is an alias for `Quantity::new` and provides a more explicit
    /// name when constructing from raw u128 values in codec contexts.
    pub fn from_u128(lots: u128) -> Self {
        Quantity::new(lots)
    }

    /// Parse a decimal string into a Quantity (in lots).
    ///
    /// This method parses a human-readable quantity string and converts it to
    /// the exact integer lot count based on the instrument's display decimals.
    ///
    /// # Arguments
    ///
    /// * `input` - The decimal string to parse (e.g., "1.5").
    /// * `spec` - The instrument specification containing display decimal info.
    ///
    /// # Returns
    ///
    /// * `Ok(Quantity)` - The parsed quantity in lots.
    /// * `Err(QuantityParseError)` - If parsing fails (overflow, inexact fraction, etc.)
    ///
    /// # Examples
    ///
    /// ```
    /// use engine_types::{InstrumentSpec, Price, Quantity};
    /// let spec = InstrumentSpec::new(1, Price::new(1), Quantity::new(1), 0, 18, None);
    /// let quantity = Quantity::from_decimal_string("1.5", &spec)?;
    /// assert_eq!(quantity.inner(), 1500000000000000000);
    /// # Ok::<(), engine_types::QuantityParseError>(())
    /// ```
    pub fn from_decimal_string(
        input: &str,
        spec: &InstrumentSpec,
    ) -> Result<Quantity, QuantityParseError> {
        quantity_from_decimal_string(input, spec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantity_new_stores_u128_lots() {
        // Given: a u128 value representing lots
        let original_lots: u128 = 1_500_000_000_000_000_000;

        // When we create a Quantity from it and extract the inner value
        let quantity = Quantity::new(original_lots);
        let extracted_lots = quantity.inner();

        // Then the round-trip preserves the value
        assert_eq!(original_lots, extracted_lots);
    }

    #[test]
    fn price_and_quantity_are_copy() {
        // Given a Quantity
        let quantity = Quantity::new(100);

        // When we assign it to another variable
        let copy = quantity;

        // Then both the original and copy can be used (Copy trait)
        assert_eq!(quantity.inner(), 100);
        assert_eq!(copy.inner(), 100);
    }
}
