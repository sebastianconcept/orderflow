//! Price tick count for an instrument.
//!
//! When a command or event names an economic price, it uses this module so ticks
//! stay a Price, not Quantity lots.

use crate::instrument_spec::{price_from_decimal_string, InstrumentSpec};
use crate::PriceParseError;

/// Price is the integer tick count of an economic price.
/// When a command or event names a price, it uses this type so the value is ticks,
/// not Quantity lots, and not a floating-point conversion.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Price(i64);

impl Price {
    /// Answers a Price from integer ticks.
    pub fn new(ticks: i64) -> Self {
        Price(ticks)
    }

    /// Answers the i64 tick count of this Price.
    pub fn inner(&self) -> i64 {
        self.0
    }

    /// Answers a Price from an i64 tick count.
    pub fn from_i64(ticks: i64) -> Self {
        Price::new(ticks)
    }

    /// Answers Price ticks of display text for an instrument.
    /// Requires the instrument display scale.
    ///
    /// ```
    /// use engine_types::{InstrumentId, InstrumentSpec, Price, Quantity};
    /// let spec = InstrumentSpec::new(
    ///     InstrumentId::new(1),
    ///     Price::new(1),
    ///     Quantity::new(1),
    ///     2,
    ///     0,
    ///     None,
    /// );
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

    /// Answers display text of this Price at the instrument scale.
    pub fn to_decimal_string(&self, spec: &InstrumentSpec) -> String {
        crate::instrument_spec::price_to_decimal_string(*self, spec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_new_stores_i64_ticks() {
        // Given an i64 value representing ticks
        let original_ticks: i64 = 10025;

        // When we create a Price from it and extract the inner value
        let price = Price::new(original_ticks);
        let extracted_ticks = price.inner();

        // Then the round-trip preserves the value
        assert_eq!(original_ticks, extracted_ticks);
    }

    #[test]
    fn price_is_copy() {
        // Given a Price
        let price = Price::new(100);

        // When we assign it to another variable
        let copy = price;

        // Then both the original and copy can be used
        assert_eq!(price.inner(), 100);
        assert_eq!(copy.inner(), 100);
    }
}
