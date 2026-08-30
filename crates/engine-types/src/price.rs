//! Price type representing ticks as i64 values.
//!
//! Price is a strongly-typed newtype around i64 that represents economic
//! prices in ticks. This ensures type safety and prevents accidental mixing
//! of price values with other integer quantities.

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
