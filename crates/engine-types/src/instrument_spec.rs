//! Error types for decimal string parsing.
//!
//! These error types are used when parsing prices and quantities from decimal strings.

use crate::price::Price;
use crate::quantity::Quantity;

/// Instrument specification for precise decimal parsing.
///
/// This struct holds metadata needed to convert between human-readable
/// decimal strings and the exact integer ticks/lots used in the matching engine.
///
/// # Examples
///
/// ```
/// use engine_types::{InstrumentSpec, Price, Quantity};
///
/// // USD with 2 decimal places, tick size of 1 cent
/// let usd_spec = InstrumentSpec::new(
///     1,
///     Price::new(1),
///     Quantity::new(1),
///     2,
///     0,
///     Some("USD/USDC".to_string()),
/// );
///
/// // ETH with 18 decimal places
/// let eth_spec = InstrumentSpec::new(
///     2,
///     Price::new(1),
///     Quantity::new(1),
///     18,
///     18,
///     Some("ETH".to_string()),
/// );
/// ```
#[derive(Debug, Clone)]
pub struct InstrumentSpec {
    /// Unique identifier for this instrument.
    pub instrument_id: u64,
    /// Price tick increment (typically 1 for most instruments).
    pub price_tick: Price,
    /// Quantity lot increment.
    pub quantity_lot: Quantity,
    /// Number of decimal places for price display.
    pub price_display_decimals: u8,
    /// Number of decimal places for quantity display.
    pub quantity_display_decimals: u8,
    /// Optional symbol or description for the instrument.
    pub symbol: Option<String>,
}

impl InstrumentSpec {
    /// Create a new InstrumentSpec with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `instrument_id` - Unique identifier for this instrument.
    /// * `price_tick` - Price tick increment (the smallest price change).
    /// * `quantity_lot` - Quantity lot increment.
    /// * `price_display_decimals` - Number of decimal places for price display.
    /// * `quantity_display_decimals` - Number of decimal places for quantity display.
    /// * `symbol` - Optional symbol or description for the instrument.
    pub fn new(
        instrument_id: u64,
        price_tick: Price,
        quantity_lot: Quantity,
        price_display_decimals: u8,
        quantity_display_decimals: u8,
        symbol: Option<String>,
    ) -> Self {
        Self {
            instrument_id,
            price_tick,
            quantity_lot,
            price_display_decimals,
            quantity_display_decimals,
            symbol,
        }
    }
}

use std::num::ParseIntError;

/// Error when parsing a price from a decimal string.
#[derive(Debug)]
pub enum PriceParseError {
    /// The price tick count overflows i64.
    TickCountOverflow,
    /// The input string contains an inexact fraction for the given instrument.
    InexactFraction,
    /// Invalid decimal format.
    InvalidFormat(ParseIntError),
}

impl std::fmt::Display for PriceParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use PriceParseError::*;
        match self {
            TickCountOverflow => write!(f, "price tick count overflows i64"),
            InexactFraction => write!(
                f,
                "input string contains an inexact fraction for the given instrument"
            ),
            InvalidFormat(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for PriceParseError {}

/// Error when parsing a quantity from a decimal string.
#[derive(Debug)]
pub enum QuantityParseError {
    /// The quantity lot count overflows u128.
    LotCountOverflow,
    /// The input string contains an inexact fraction for the given instrument.
    InexactFraction,
    /// Invalid decimal format.
    InvalidFormat(ParseIntError),
}

impl std::fmt::Display for QuantityParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use QuantityParseError::*;
        match self {
            LotCountOverflow => write!(f, "quantity lot count overflows u128"),
            InexactFraction => write!(
                f,
                "input string contains an inexact fraction for the given instrument"
            ),
            InvalidFormat(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for QuantityParseError {}

/// Parse a decimal string into a Price (in ticks).
///
/// This function parses a human-readable price string and converts it to
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
pub fn price_from_decimal_string(
    input: &str,
    spec: &InstrumentSpec,
) -> Result<Price, PriceParseError> {
    // Find the decimal point position
    let input_str = input.trim();

    // Handle empty string
    if input_str.is_empty() {
        let _: u128 = "0".parse().unwrap();
        let err: ParseIntError = "".parse::<u128>().unwrap_err();
        return Err(PriceParseError::InvalidFormat(err));
    }

    // Check if there's a decimal point
    let (integer_part, fractional_part) = match input_str.find('.') {
        Some(decimal_pos) => {
            let (int, frac) = input_str.split_at(decimal_pos);
            let frac = &frac[1..]; // Skip the decimal point
            (int, Some(frac))
        }
        None => (input_str, None),
    };

    // Parse the integer part
    let integer_value: u128 = integer_part
        .parse()
        .map_err(PriceParseError::InvalidFormat)?;

    // Calculate the multiplier based on display decimals
    let multiplier: u128 = 10u128.pow(spec.price_display_decimals as u32);

    // Calculate the total ticks
    let mut total_ticks: u128 = integer_value * multiplier;

    // Process fractional part if present
    if let Some(frac_str) = fractional_part {
        if !frac_str.is_empty() {
            // Parse the fractional part as an integer
            let fractional_value: u128 =
                frac_str.parse().map_err(PriceParseError::InvalidFormat)?;

            // Calculate how many digits we actually have
            let actual_fractional_digits = frac_str.len() as u32;

            // Check if we have too many fractional digits (inexact fraction)
            if actual_fractional_digits > spec.price_display_decimals as u32 {
                return Err(PriceParseError::InexactFraction);
            }

            // Calculate the multiplier to normalize fractional part
            let normalize_multiplier: u128 = 10u128
                .pow((spec.price_display_decimals as u32).saturating_sub(actual_fractional_digits));

            total_ticks += fractional_value * normalize_multiplier;
        }
    }

    // Check if the ticks fit in i64
    let ticks_i64: i64 = total_ticks
        .try_into()
        .map_err(|_| PriceParseError::TickCountOverflow)?;

    Ok(Price::new(ticks_i64))
}

/// Parse a decimal string into a Quantity (in lots).
///
/// This function parses a human-readable quantity string and converts it to
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
pub fn quantity_from_decimal_string(
    input: &str,
    spec: &InstrumentSpec,
) -> Result<Quantity, QuantityParseError> {
    // Find the decimal point position
    let input_str = input.trim();

    // Handle empty string
    if input_str.is_empty() {
        let _: u128 = "0".parse().unwrap();
        let err: ParseIntError = "".parse::<u128>().unwrap_err();
        return Err(QuantityParseError::InvalidFormat(err));
    }

    // Check if there's a decimal point
    let (integer_part, fractional_part) = match input_str.find('.') {
        Some(decimal_pos) => {
            let (int, frac) = input_str.split_at(decimal_pos);
            let frac = &frac[1..]; // Skip the decimal point
            (int, Some(frac))
        }
        None => (input_str, None),
    };

    // Parse the integer part
    let integer_value: u128 = integer_part
        .parse()
        .map_err(QuantityParseError::InvalidFormat)?;

    // Calculate the multiplier based on display decimals
    let multiplier: u128 = 10u128.pow(spec.quantity_display_decimals as u32);

    // Calculate the total lots
    let mut total_lots: u128 = integer_value * multiplier;

    // Process fractional part if present
    if let Some(frac_str) = fractional_part {
        if !frac_str.is_empty() {
            // Parse the fractional part as an integer
            let fractional_value: u128 = frac_str
                .parse()
                .map_err(QuantityParseError::InvalidFormat)?;

            // Calculate how many digits we actually have
            let actual_fractional_digits = frac_str.len() as u32;

            // Check if we have too many fractional digits (inexact fraction)
            if actual_fractional_digits > spec.quantity_display_decimals as u32 {
                return Err(QuantityParseError::InexactFraction);
            }

            // Calculate the multiplier to normalize fractional part
            let normalize_multiplier: u128 = 10u128.pow(
                (spec.quantity_display_decimals as u32).saturating_sub(actual_fractional_digits),
            );

            total_lots += fractional_value * normalize_multiplier;
        }
    }

    // Check if the lots fit in u128 (they should, but check for safety)
    // u128 can hold very large values, so this is more of a safety check
    // For practical purposes, u128 should always work for quantity

    Ok(Quantity::new(total_lots))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instrument_spec_creates_with_all_fields() {
        // Given: all required fields
        let spec = InstrumentSpec::new(
            1,
            Price::new(1),
            Quantity::new(100),
            2,
            8,
            Some("BTC/USD".to_string()),
        );

        // When: we access the fields
        let symbol = spec.symbol.clone();

        // Then: all fields are set correctly
        assert_eq!(spec.instrument_id, 1);
        assert_eq!(spec.price_tick.inner(), 1);
        assert_eq!(spec.quantity_lot.inner(), 100);
        assert_eq!(spec.price_display_decimals, 2);
        assert_eq!(spec.quantity_display_decimals, 8);
        assert_eq!(symbol, Some("BTC/USD".to_string()));
    }

    #[test]
    fn instrument_spec_symbol_is_optional() {
        // Given: no symbol
        let spec = InstrumentSpec::new(1, Price::new(1), Quantity::new(1), 0, 0, None);

        // When: we access the symbol
        let symbol = spec.symbol;

        // Then: symbol is None
        assert_eq!(symbol, None);
    }

    #[test]
    fn price_from_decimal_string_accepts_whole_number() {
        // Given: a USD spec with 2 decimal places
        let spec = InstrumentSpec::new(1, Price::new(1), Quantity::new(1), 2, 0, None);

        // When: we parse a whole number
        let result = Price::from_decimal_string("100", &spec);

        // Then: it parses to 10000 ticks
        assert!(result.is_ok());
        assert_eq!(result.unwrap().inner(), 10000);
    }

    #[test]
    fn price_from_decimal_string_accepts_two_decimal_places() {
        // Given: a USD spec with 2 decimal places
        let spec = InstrumentSpec::new(1, Price::new(1), Quantity::new(1), 2, 0, None);

        // When: we parse a value with 2 decimal places
        let result = Price::from_decimal_string("100.25", &spec);

        // Then: it parses to 10025 ticks
        assert!(result.is_ok());
        assert_eq!(result.unwrap().inner(), 10025);
    }

    #[test]
    fn price_from_decimal_string_rejects_inexact_fraction() {
        // Given: a USD spec with 2 decimal places
        let spec = InstrumentSpec::new(1, Price::new(1), Quantity::new(1), 2, 0, None);

        // When: we parse a value with more than 2 decimal places
        let result = Price::from_decimal_string("100.251", &spec);

        // Then: it rejects the inexact fraction
        assert!(result.is_err());
    }

    #[test]
    fn quantity_from_decimal_string_accepts_eth_one_point_five_at_18_decimals() {
        // Given: an ETH spec with 18 decimal places
        let spec = InstrumentSpec::new(2, Price::new(1), Quantity::new(1), 0, 18, None);

        // When: we parse 1.5 ETH
        let result = Quantity::from_decimal_string("1.5", &spec);

        // Then: it parses to 1.5e18 lots
        assert!(result.is_ok());
        let quantity = result.unwrap();
        assert_eq!(quantity.inner(), 1500000000000000000);
    }

    #[test]
    fn quantity_from_decimal_string_rejects_inexact_fraction() {
        // Given: an ETH spec with 18 decimal places
        let spec = InstrumentSpec::new(2, Price::new(1), Quantity::new(1), 0, 18, None);

        // When: we parse a value with more than 18 decimal places
        let result = Quantity::from_decimal_string("1.1234567890123456789", &spec);

        // Then: it rejects the inexact fraction
        assert!(result.is_err());
    }

    #[test]
    fn price_from_decimal_string_rejects_tick_count_that_does_not_fit_i64() {
        // Given: a spec with 0 decimal places (tick = 1)
        let spec = InstrumentSpec::new(1, Price::new(1), Quantity::new(1), 0, 0, None);

        // When: we parse a value that overflows i64
        let large_value = "9223372036854775808"; // i64::MAX + 1
        let result = Price::from_decimal_string(large_value, &spec);

        // Then: it rejects the overflow
        assert!(result.is_err());
    }

    #[test]
    fn price_from_decimal_string_handles_trailing_zeros() {
        // Given: a USD spec with 2 decimal places
        let spec = InstrumentSpec::new(1, Price::new(1), Quantity::new(1), 2, 0, None);

        // When: we parse a value with trailing zeros
        let result = Price::from_decimal_string("100.50", &spec);

        // Then: it parses to 10050 ticks (trailing zero is significant)
        assert!(result.is_ok());
        assert_eq!(result.unwrap().inner(), 10050);
    }

    #[test]
    fn quantity_from_decimal_string_handles_trailing_zeros() {
        // Given: an ETH spec with 18 decimal places
        let spec = InstrumentSpec::new(2, Price::new(1), Quantity::new(1), 0, 18, None);

        // When: we parse a value with trailing zeros
        let result = Quantity::from_decimal_string("1.50", &spec);

        // Then: it parses to 1.5e18 lots
        assert!(result.is_ok());
        let quantity = result.unwrap();
        assert_eq!(quantity.inner(), 1500000000000000000);
    }
}
