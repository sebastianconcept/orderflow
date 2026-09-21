//! Display scale and exact decimal parse for one instrument.
//!
//! When a caller converts a decimal string into integer ticks or lots, it uses
//! this module so economic values stay exact integers.

use displaydoc::Display;
use thiserror::Error;

use crate::identity::InstrumentId;
use crate::price::Price;
use crate::quantity::Quantity;

/// InstrumentSpec is the display scale and tick or lot increment for one instrument.
/// When a caller needs how many fractional digits a price or quantity uses, it uses
/// this type so scale lives on the instrument, not on Price or Quantity.
///
/// # Examples
///
/// ```
/// use engine_types::{InstrumentId, InstrumentSpec, Price, Quantity};
///
/// let usd_spec = InstrumentSpec::new(
///     InstrumentId::new(1),
///     Price::new(1),
///     Quantity::new(1),
///     2,
///     0,
///     Some("USD/USDC".to_string()),
/// );
///
/// let eth_spec = InstrumentSpec::new(
///     InstrumentId::new(2),
///     Price::new(1),
///     Quantity::new(1),
///     18,
///     18,
///     Some("ETH".to_string()),
/// );
/// ```
#[derive(Debug, Clone)]
pub struct InstrumentSpec {
    pub instrument_id: InstrumentId,
    pub price_tick: Price,
    pub quantity_lot: Quantity,
    pub price_display_decimals: u8,
    pub quantity_display_decimals: u8,
    pub symbol: Option<String>,
}

impl InstrumentSpec {
    /// Answers an InstrumentSpec from its fields.
    pub fn new(
        instrument_id: InstrumentId,
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

/// PriceParseError is the failure of a decimal string to become a Price.
/// Overflow, inexact fraction, empty input, and invalid text are distinct.
#[derive(Debug, Display, Error, PartialEq, Eq)]
pub enum PriceParseError {
    /// Price tick count overflows i64
    TickCountOverflow,
    /// Input string contains an inexact fraction for the given instrument
    InexactFraction,
    /// Decimal string is empty
    EmptyInput,
    /// Invalid decimal format
    InvalidFormat,
}

/// QuantityParseError is the failure of a decimal string to become a Quantity.
/// Overflow, inexact fraction, empty input, and invalid text are distinct.
#[derive(Debug, Display, Error, PartialEq, Eq)]
pub enum QuantityParseError {
    /// Quantity lot count overflows u128
    LotCountOverflow,
    /// Input string contains an inexact fraction for the given instrument
    InexactFraction,
    /// Decimal string is empty
    EmptyInput,
    /// Invalid decimal format
    InvalidFormat,
}

/// Shared failure of scaled decimal parse.
/// Overflow, inexact fraction, empty input, and invalid text are distinct.
enum DecimalParseOutcome {
    /// Scaled integer exceeds the storage width of the target type.
    Overflow,
    /// Fractional digits exceed the instrument display scale.
    InexactFraction,
    /// Input string is empty or whitespace only.
    EmptyInput,
    /// Input string is not a valid decimal number.
    InvalidFormat,
}

/// Answers the scaled integer of a non-negative decimal string at a fixed scale.
fn parse_unsigned_decimal_magnitude(
    input: &str,
    display_decimals: u8,
) -> Result<u128, DecimalParseOutcome> {
    if input.is_empty() {
        return Err(DecimalParseOutcome::EmptyInput);
    }

    let (integer_part, fractional_part) = match input.find('.') {
        Some(decimal_pos) => {
            let (integer_text, fractional_with_dot) = input.split_at(decimal_pos);
            let fractional_text = &fractional_with_dot[1..];
            (integer_text, Some(fractional_text))
        }
        None => (input, None),
    };

    if integer_part.is_empty() {
        return Err(DecimalParseOutcome::InvalidFormat);
    }

    let integer_value: u128 = integer_part
        .parse()
        .map_err(|_| DecimalParseOutcome::InvalidFormat)?;

    let multiplier = 10u128
        .checked_pow(display_decimals as u32)
        .ok_or(DecimalParseOutcome::Overflow)?;

    let mut total = integer_value
        .checked_mul(multiplier)
        .ok_or(DecimalParseOutcome::Overflow)?;

    if let Some(fractional_text) = fractional_part {
        if !fractional_text.is_empty() {
            let fractional_value: u128 = fractional_text
                .parse()
                .map_err(|_| DecimalParseOutcome::InvalidFormat)?;
            let actual_fractional_digits = fractional_text.len() as u32;
            if actual_fractional_digits > display_decimals as u32 {
                return Err(DecimalParseOutcome::InexactFraction);
            }
            let normalize_multiplier = 10u128
                .checked_pow((display_decimals as u32).saturating_sub(actual_fractional_digits))
                .ok_or(DecimalParseOutcome::Overflow)?;
            let fractional_scaled = fractional_value
                .checked_mul(normalize_multiplier)
                .ok_or(DecimalParseOutcome::Overflow)?;
            total = total
                .checked_add(fractional_scaled)
                .ok_or(DecimalParseOutcome::Overflow)?;
        }
    }

    Ok(total)
}

/// Answers the scaled integer and sign of a decimal string that may be negative.
fn parse_signed_decimal_magnitude(
    input: &str,
    display_decimals: u8,
) -> Result<(u128, bool), DecimalParseOutcome> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(DecimalParseOutcome::EmptyInput);
    }

    let (unsigned_text, negative) = if let Some(rest) = trimmed.strip_prefix('-') {
        if rest.is_empty() {
            return Err(DecimalParseOutcome::InvalidFormat);
        }
        (rest, true)
    } else if let Some(rest) = trimmed.strip_prefix('+') {
        if rest.is_empty() {
            return Err(DecimalParseOutcome::InvalidFormat);
        }
        (rest, false)
    } else {
        (trimmed, false)
    };

    let magnitude = parse_unsigned_decimal_magnitude(unsigned_text, display_decimals)?;
    Ok((magnitude, negative))
}

impl PriceParseError {
    /// Answers the PriceParseError for a DecimalParseOutcome.
    fn from_decimal_parse_outcome(outcome: DecimalParseOutcome) -> Self {
        match outcome {
            DecimalParseOutcome::Overflow => PriceParseError::TickCountOverflow,
            DecimalParseOutcome::InexactFraction => PriceParseError::InexactFraction,
            DecimalParseOutcome::EmptyInput => PriceParseError::EmptyInput,
            DecimalParseOutcome::InvalidFormat => PriceParseError::InvalidFormat,
        }
    }
}

impl QuantityParseError {
    /// Answers the QuantityParseError for a DecimalParseOutcome.
    fn from_decimal_parse_outcome(outcome: DecimalParseOutcome) -> Self {
        match outcome {
            DecimalParseOutcome::Overflow => QuantityParseError::LotCountOverflow,
            DecimalParseOutcome::InexactFraction => QuantityParseError::InexactFraction,
            DecimalParseOutcome::EmptyInput => QuantityParseError::EmptyInput,
            DecimalParseOutcome::InvalidFormat => QuantityParseError::InvalidFormat,
        }
    }
}

/// Answers the decimal string of a magnitude at a fixed scale.
fn format_unsigned_decimal(magnitude: u128, display_decimals: u8, negative: bool) -> String {
    if display_decimals == 0 {
        return if negative {
            format!("-{magnitude}")
        } else {
            magnitude.to_string()
        };
    }

    let digits = magnitude.to_string();
    let decimal_count = display_decimals as usize;
    let body = if digits.len() <= decimal_count {
        let pad = decimal_count - digits.len();
        format!("0.{}{}", "0".repeat(pad), digits)
    } else {
        let split = digits.len() - decimal_count;
        format!("{}.{}", &digits[..split], &digits[split..])
    };

    if negative {
        format!("-{body}")
    } else {
        body
    }
}

/// Answers Price ticks of a decimal string for an instrument.
///
/// # Examples
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
pub fn price_from_decimal_string(
    input: &str,
    spec: &InstrumentSpec,
) -> Result<Price, PriceParseError> {
    let (magnitude, negative) = parse_signed_decimal_magnitude(input, spec.price_display_decimals)
        .map_err(PriceParseError::from_decimal_parse_outcome)?;

    let ticks_i64: i64 = magnitude
        .try_into()
        .map_err(|_| PriceParseError::TickCountOverflow)?;
    let ticks_i64 = if negative {
        ticks_i64
            .checked_neg()
            .ok_or(PriceParseError::TickCountOverflow)?
    } else {
        ticks_i64
    };

    let tick_increment = spec.price_tick.inner();
    if tick_increment != 0 && ticks_i64.rem_euclid(tick_increment) != 0 {
        return Err(PriceParseError::InexactFraction);
    }

    Ok(Price::new(ticks_i64))
}

/// Answers Quantity lots of a decimal string for an instrument.
///
/// # Examples
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
pub fn quantity_from_decimal_string(
    input: &str,
    spec: &InstrumentSpec,
) -> Result<Quantity, QuantityParseError> {
    let trimmed = input.trim();
    if trimmed.starts_with('-') {
        return Err(QuantityParseError::InvalidFormat);
    }

    let magnitude = parse_unsigned_decimal_magnitude(
        trimmed.strip_prefix('+').unwrap_or(trimmed),
        spec.quantity_display_decimals,
    )
    .map_err(QuantityParseError::from_decimal_parse_outcome)?;

    let lot_increment = spec.quantity_lot.inner();
    if lot_increment != 0 && magnitude % lot_increment != 0 {
        return Err(QuantityParseError::InexactFraction);
    }

    Ok(Quantity::new(magnitude))
}

/// Answers display text of a Price at the instrument scale.
pub fn price_to_decimal_string(price: Price, spec: &InstrumentSpec) -> String {
    let ticks = price.inner();
    let negative = ticks < 0;
    let magnitude = ticks.unsigned_abs() as u128;
    format_unsigned_decimal(magnitude, spec.price_display_decimals, negative)
}

/// Answers display text of a Quantity at the instrument scale.
pub fn quantity_to_decimal_string(quantity: Quantity, spec: &InstrumentSpec) -> String {
    format_unsigned_decimal(quantity.inner(), spec.quantity_display_decimals, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instrument_spec_creates_with_all_fields() {
        // Given all required fields
        let spec = InstrumentSpec::new(
            InstrumentId::new(1),
            Price::new(1),
            Quantity::new(100),
            2,
            8,
            Some("BTC/USD".to_string()),
        );

        // When we access the fields
        let symbol = spec.symbol.clone();

        // Then all fields are set correctly
        assert_eq!(spec.instrument_id, InstrumentId::new(1));
        assert_eq!(spec.price_tick.inner(), 1);
        assert_eq!(spec.quantity_lot.inner(), 100);
        assert_eq!(spec.price_display_decimals, 2);
        assert_eq!(spec.quantity_display_decimals, 8);
        assert_eq!(symbol, Some("BTC/USD".to_string()));
    }

    #[test]
    fn instrument_spec_symbol_is_optional() {
        // Given no symbol
        let spec = InstrumentSpec::new(
            InstrumentId::new(1),
            Price::new(1),
            Quantity::new(1),
            0,
            0,
            None,
        );

        // When we access the symbol
        let symbol = spec.symbol;

        // Then symbol is None
        assert_eq!(symbol, None);
    }

    #[test]
    fn price_from_decimal_string_accepts_whole_number() {
        // Given a USD spec with 2 decimal places
        let spec = InstrumentSpec::new(
            InstrumentId::new(1),
            Price::new(1),
            Quantity::new(1),
            2,
            0,
            None,
        );

        // When we parse a whole number
        let result = Price::from_decimal_string("100", &spec);

        // Then it parses to 10000 ticks
        assert!(result.is_ok());
        assert_eq!(result.unwrap().inner(), 10000);
    }

    #[test]
    fn price_from_decimal_string_accepts_two_decimal_places() {
        // Given a USD spec with 2 decimal places
        let spec = InstrumentSpec::new(
            InstrumentId::new(1),
            Price::new(1),
            Quantity::new(1),
            2,
            0,
            None,
        );

        // When we parse a value with 2 decimal places
        let result = Price::from_decimal_string("100.25", &spec);

        // Then it parses to 10025 ticks
        assert!(result.is_ok());
        assert_eq!(result.unwrap().inner(), 10025);
    }

    #[test]
    fn price_from_decimal_string_rejects_inexact_fraction() {
        // Given a USD spec with 2 decimal places
        let spec = InstrumentSpec::new(
            InstrumentId::new(1),
            Price::new(1),
            Quantity::new(1),
            2,
            0,
            None,
        );

        // When we parse a value with more than 2 decimal places
        let result = Price::from_decimal_string("100.251", &spec);

        // Then it rejects the inexact fraction
        assert!(result.is_err());
    }

    #[test]
    fn quantity_from_decimal_string_accepts_eth_one_point_five_at_18_decimals() {
        // Given an ETH spec with 18 decimal places
        let spec = InstrumentSpec::new(
            InstrumentId::new(2),
            Price::new(1),
            Quantity::new(1),
            0,
            18,
            None,
        );

        // When we parse 1.5 ETH
        let result = Quantity::from_decimal_string("1.5", &spec);

        // Then it parses to 1.5e18 lots
        assert!(result.is_ok());
        let quantity = result.unwrap();
        assert_eq!(quantity.inner(), 1500000000000000000);
    }

    #[test]
    fn quantity_from_decimal_string_rejects_inexact_fraction() {
        // Given an ETH spec with 18 decimal places
        let spec = InstrumentSpec::new(
            InstrumentId::new(2),
            Price::new(1),
            Quantity::new(1),
            0,
            18,
            None,
        );

        // When we parse a value with more than 18 decimal places
        let result = Quantity::from_decimal_string("1.1234567890123456789", &spec);

        // Then it rejects the inexact fraction
        assert!(result.is_err());
    }

    #[test]
    fn price_from_decimal_string_rejects_tick_count_that_does_not_fit_i64() {
        // Given a spec with 0 decimal places (tick = 1)
        let spec = InstrumentSpec::new(
            InstrumentId::new(1),
            Price::new(1),
            Quantity::new(1),
            0,
            0,
            None,
        );

        // When we parse a value that overflows i64
        let large_value = "9223372036854775808"; // i64::MAX + 1
        let result = Price::from_decimal_string(large_value, &spec);

        // Then it rejects the overflow
        assert!(result.is_err());
    }

    #[test]
    fn price_from_decimal_string_handles_trailing_zeros() {
        // Given a USD spec with 2 decimal places
        let spec = InstrumentSpec::new(
            InstrumentId::new(1),
            Price::new(1),
            Quantity::new(1),
            2,
            0,
            None,
        );

        // When we parse a value with trailing zeros
        let result = Price::from_decimal_string("100.50", &spec);

        // Then it parses to 10050 ticks (trailing zero is significant)
        assert!(result.is_ok());
        assert_eq!(result.unwrap().inner(), 10050);
    }

    #[test]
    fn quantity_from_decimal_string_handles_trailing_zeros() {
        // Given an ETH spec with 18 decimal places
        let spec = InstrumentSpec::new(
            InstrumentId::new(2),
            Price::new(1),
            Quantity::new(1),
            0,
            18,
            None,
        );

        // When we parse a value with trailing zeros
        let result = Quantity::from_decimal_string("1.50", &spec);

        // Then it parses to 1.5e18 lots
        assert!(result.is_ok());
        let quantity = result.unwrap();
        assert_eq!(quantity.inner(), 1500000000000000000);
    }

    #[test]
    fn price_from_decimal_string_rejects_empty_input() {
        // Given a USD spec and an empty decimal string
        let spec = InstrumentSpec::new(
            InstrumentId::new(1),
            Price::new(1),
            Quantity::new(1),
            2,
            0,
            None,
        );

        // When the empty string is parsed as a price
        let result = Price::from_decimal_string("", &spec);

        // Then parsing fails with empty input
        assert!(matches!(result, Err(PriceParseError::EmptyInput)));
    }

    #[test]
    fn price_from_decimal_string_rejects_display_decimals_that_overflow_u128_pow() {
        // Given a spec whose price display decimals exceed u128::MAX as a power of ten
        let spec = InstrumentSpec::new(
            InstrumentId::new(1),
            Price::new(1),
            Quantity::new(1),
            40,
            0,
            None,
        );

        // When a whole number is parsed
        let result = Price::from_decimal_string("1", &spec);

        // Then parsing fails with tick overflow instead of panicking
        assert!(matches!(result, Err(PriceParseError::TickCountOverflow)));
    }

    #[test]
    fn price_from_decimal_string_rejects_integer_part_that_overflows_u128_before_i64_check() {
        // Given a spec with 20 display decimals so 10^20 times a large integer overflows u128
        let spec = InstrumentSpec::new(
            InstrumentId::new(1),
            Price::new(1),
            Quantity::new(1),
            20,
            0,
            None,
        );

        // When a 20-digit integer is parsed
        let result = Price::from_decimal_string("10000000000000000000", &spec);

        // Then parsing fails with tick overflow
        assert!(matches!(result, Err(PriceParseError::TickCountOverflow)));
    }

    #[test]
    fn price_from_decimal_string_accepts_negative_ticks() {
        // Given a USD spec with 2 decimal places
        let spec = InstrumentSpec::new(
            InstrumentId::new(1),
            Price::new(1),
            Quantity::new(1),
            2,
            0,
            None,
        );

        // When a negative decimal string is parsed
        let result = Price::from_decimal_string("-100.25", &spec);

        // Then the price is negative ticks
        assert_eq!(result.expect("negative price should parse").inner(), -10025);
    }

    #[test]
    fn quantity_from_decimal_string_rejects_lot_count_overflow() {
        // Given a spec whose quantity display decimals overflow u128 as a power of ten
        let spec = InstrumentSpec::new(
            InstrumentId::new(2),
            Price::new(1),
            Quantity::new(1),
            0,
            40,
            None,
        );

        // When a whole number is parsed as quantity
        let result = Quantity::from_decimal_string("1", &spec);

        // Then parsing fails with lot overflow instead of panicking
        assert!(matches!(result, Err(QuantityParseError::LotCountOverflow)));
    }

    #[test]
    fn quantity_from_decimal_string_rejects_negative_input() {
        // Given an ETH spec
        let spec = InstrumentSpec::new(
            InstrumentId::new(2),
            Price::new(1),
            Quantity::new(1),
            0,
            18,
            None,
        );

        // When a negative quantity string is parsed
        let result = Quantity::from_decimal_string("-1.5", &spec);

        // Then parsing fails
        assert!(matches!(result, Err(QuantityParseError::InvalidFormat)));
    }

    #[test]
    fn price_to_decimal_string_round_trips_usd_two_decimals() {
        // Given a USD spec and a parsed price
        let spec = InstrumentSpec::new(
            InstrumentId::new(1),
            Price::new(1),
            Quantity::new(1),
            2,
            0,
            None,
        );
        let price = Price::from_decimal_string("100.25", &spec).expect("parse");

        // When the price is formatted
        let formatted = price.to_decimal_string(&spec);

        // Then the decimal string matches the original scale
        assert_eq!(formatted, "100.25");
    }

    #[test]
    fn quantity_to_decimal_string_round_trips_eth_eighteen_decimals() {
        // Given an ETH spec and 1.5 lots at 18 decimals
        let spec = InstrumentSpec::new(
            InstrumentId::new(2),
            Price::new(1),
            Quantity::new(1),
            0,
            18,
            None,
        );
        let quantity = Quantity::from_decimal_string("1.5", &spec).expect("parse");

        // When the quantity is formatted
        let formatted = quantity.to_decimal_string(&spec);

        // Then the fractional part has 18 digits
        assert_eq!(formatted, "1.500000000000000000");
    }
}
