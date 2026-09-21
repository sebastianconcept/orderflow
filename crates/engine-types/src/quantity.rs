//! Quantity lot count for an instrument.
//!
//! When a command or event names an economic size, it uses this module so lots
//! stay a Quantity, not Price ticks.

use crate::instrument_spec::{quantity_from_decimal_string, InstrumentSpec};
use crate::QuantityParseError;

/// Quantity is the integer lot count of an economic quantity.
/// When a command or event names a size, it uses this type so the value is lots,
/// not Price ticks, and not a floating-point conversion.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Quantity(u128);

impl Quantity {
    /// Answers a Quantity from integer lots.
    pub fn new(lots: u128) -> Self {
        Quantity(lots)
    }

    /// Answers the u128 lot count of this Quantity.
    pub fn inner(&self) -> u128 {
        self.0
    }

    /// Answers a Quantity from a u128 lot count.
    pub fn from_u128(lots: u128) -> Self {
        Quantity::new(lots)
    }

    /// Answers Quantity lots of display text for an instrument.
    /// Requires the instrument display scale.
    ///
    /// ```
    /// use engine_types::{InstrumentId, InstrumentSpec, Price, Quantity};
    /// let spec = InstrumentSpec::new(
    ///     InstrumentId::new(1),
    ///     Price::new(1),
    ///     Quantity::new(1),
    ///     0,
    ///     18,
    ///     None,
    /// );
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

    /// Answers display text of this Quantity at the instrument scale.
    pub fn to_decimal_string(&self, spec: &InstrumentSpec) -> String {
        crate::instrument_spec::quantity_to_decimal_string(*self, spec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantity_new_stores_u128_lots() {
        // Given a u128 value representing lots
        let original_lots: u128 = 1_500_000_000_000_000_000;

        // When we create a Quantity from it and extract the inner value
        let quantity = Quantity::new(original_lots);
        let extracted_lots = quantity.inner();

        // Then the round-trip preserves the value
        assert_eq!(original_lots, extracted_lots);
    }

    #[test]
    fn quantity_is_copy() {
        // Given a Quantity
        let quantity = Quantity::new(100);

        // When we assign it to another variable
        let copy = quantity;

        // Then both the original and copy can be used
        assert_eq!(quantity.inner(), 100);
        assert_eq!(copy.inner(), 100);
    }
}
