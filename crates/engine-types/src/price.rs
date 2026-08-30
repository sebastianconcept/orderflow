//! Price type representing ticks as i64 values.
//!
//! Price is a strongly-typed newtype around i64 that represents economic
//! prices in ticks. This ensures type safety and prevents accidental mixing
//! of price values with other integer quantities.

use crate::instrument_spec::{price_from_decimal_string, InstrumentSpec};
use crate::PriceParseError;

/// Price in ticks (i64).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Price(i64);

impl Price {
    /// Create a new Price from ticks.
    pub fn new(ticks: i64) -> Self {
        Price(ticks)
    }

    /// Get the inner ticks value.
    pub fn inner(&self) -> i64 {
        self.0
    }

    /// Create a new Price from an i64 value.
    ///
    /// This is an alias for `Price::new` and provides a more explicit
    /// name when constructing from raw i64 values in codec contexts.
    pub fn from_i64(ticks: i64) -> Self {
        Price::new(ticks)
    }

    /// Parse a decimal string into a Price (in ticks).
    ///
    /// This method parses a human-readable price string and converts it to
    /// the exact integer tick count based on the instrument's display decimals.
    ///
    /// # Arguments
    ///
    /// * `input` - The decimal string to parse (e.g., "100.25").
    /// * `spec` - The instrument specification containing display decimal info.
    ///
    /// # Returns
    ///
    /// * `Ok(Price)` - The parsed price in ticks.
    /// * `Err(PriceParseError)` - If parsing fails (overflow, inexact fraction, etc.)
    ///
    /// # Examples
    ///
    /// ```
    /// use engine_types::{InstrumentSpec, Price, Quantity};
    /// let spec = InstrumentSpec::new(1, Price::new(1), Quantity::new(1), 2, 0, None);
    /// let price = Price::from_decimal_string("100.25", &spec)?;
    /// assert_eq!(price.inner(), 10025);
    /// # Ok::<(), engine_types::PriceParseError>(())
    /// ```
    pub fn from_decimal_string(
        input: &str,
        spec: &InstrumentSpec,
    ) -> Result<Price, PriceParseError> {
        price_from_decimal_string(input, spec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_new_stores_i64_ticks() {
        // Given: a i64 value representing ticks
        let original_ticks: i64 = 10025;

        // When we create a Price from it and extract the inner value
        let price = Price::new(original_ticks);
        let extracted_ticks = price.inner();

        // Then the round-trip preserves the value
        assert_eq!(original_ticks, extracted_ticks);
    }

    #[test]
    fn price_and_quantity_are_copy() {
        // Given a Price
        let price = Price::new(100);

        // When we assign it to another variable
        let copy = price;

        // Then both the original and copy can be used (Copy trait)
        assert_eq!(price.inner(), 100);
        assert_eq!(copy.inner(), 100);
    }
}
