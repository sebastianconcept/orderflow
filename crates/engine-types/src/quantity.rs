//! Quantity type representing lots as u128 values.
//!
//! Quantity is a strongly-typed newtype around u128 that represents
//! economic quantities in lots. This ensures type safety and prevents
//! accidental mixing of quantity values with other integer types.

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
