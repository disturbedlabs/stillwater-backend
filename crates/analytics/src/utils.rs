use rust_decimal::Decimal;
use rust_decimal::prelude::*;

/// Check if current tick is within position's range
pub fn is_in_range(current_tick: i32, tick_lower: i32, tick_upper: i32) -> bool {
    current_tick >= tick_lower && current_tick < tick_upper
}

/// Calculate distance to the nearest range edge
pub fn distance_to_range_edge(current_tick: i32, tick_lower: i32, tick_upper: i32) -> i32 {
    if current_tick < tick_lower {
        return 0; // Out of range below
    }
    if current_tick >= tick_upper {
        return 0; // Out of range above
    }

    let dist_to_lower = current_tick - tick_lower;
    let dist_to_upper = tick_upper - current_tick;

    dist_to_lower.min(dist_to_upper)
}

/// Convert tick to sqrt price using Uniswap v3/v4 formula: sqrt_price = 1.0001^(tick/2)
///
/// Mathematical derivation:
/// - In Uniswap v3/v4, price P = (sqrt_price)^2
/// - sqrt_price = 1.0001^(tick/2)
/// - Therefore: price = 1.0001^tick
///
/// We use sqrt_price internally for accuracy in liquidity calculations
pub fn tick_to_sqrt_price(tick: i32) -> Decimal {
    // sqrt_price = 1.0001^(tick/2) = e^(tick/2 * ln(1.0001))
    // ln(1.0001) ≈ 0.00009999500033330834
    let ln_base = Decimal::from_str("0.00009999500033330834").unwrap();

    // Calculate tick/2 * ln(1.0001)
    let tick_decimal = Decimal::from(tick);
    let half = Decimal::from_str("0.5").unwrap();
    let exponent = tick_decimal * half * ln_base;

    // For safety, cap the result to avoid overflow
    if exponent.abs() > Decimal::from(100) {
        if tick > 0 {
            Decimal::from_str("1000000").unwrap() // sqrt of 1 trillion
        } else {
            Decimal::from_str("0.000001").unwrap() // sqrt of 1 trillionth
        }
    } else {
        exponent.exp()
    }
}

/// Convert tick to price using Uniswap v3/v4 formula: price = 1.0001^tick
pub fn tick_to_price(tick: i32) -> Decimal {
    let sqrt_price = tick_to_sqrt_price(tick);
    sqrt_price * sqrt_price
}

/// Convert price to tick (inverse of tick_to_price)
pub fn price_to_tick(price: Decimal) -> i32 {
    if price <= Decimal::ZERO {
        return 0;
    }

    // tick = log(price) / log(1.0001)
    // Using approximation for now
    let log_price = price.ln();
    let log_base = Decimal::from_str("1.0001").unwrap().ln();

    (log_price / log_base).round().to_i32().unwrap_or(0)
}

/// Calculate range width as a percentage
pub fn range_width_percent(tick_lower: i32, tick_upper: i32) -> Decimal {
    let price_lower = tick_to_price(tick_lower);
    let price_upper = tick_to_price(tick_upper);

    if price_lower.is_zero() {
        return Decimal::ZERO;
    }

    ((price_upper - price_lower) / price_lower) * Decimal::from(100)
}

/// Calculate token amounts from liquidity at a given price
///
/// Based on Uniswap v3 whitepaper equations 6.29 and 6.30:
///
/// When current price P is in range [Pa, Pb]:
///   - x = L * (√Pb - √P) / (√P * √Pb)
///   - y = L * (√P - √Pa)
///
/// When P < Pa (price below range):
///   - x = L * (√Pb - √Pa) / (√Pa * √Pb)
///   - y = 0
///
/// When P > Pb (price above range):
///   - x = 0
///   - y = L * (√Pb - √Pa)
///
/// Returns: (amount0, amount1) where amount0 is token0 and amount1 is token1
pub fn get_token_amounts_from_liquidity(
    liquidity: Decimal,
    current_tick: i32,
    tick_lower: i32,
    tick_upper: i32,
) -> (Decimal, Decimal) {
    if liquidity.is_zero() {
        return (Decimal::ZERO, Decimal::ZERO);
    }

    let sqrt_price = tick_to_sqrt_price(current_tick);
    let sqrt_price_lower = tick_to_sqrt_price(tick_lower);
    let sqrt_price_upper = tick_to_sqrt_price(tick_upper);

    // Price below range: only token0
    if current_tick < tick_lower {
        let amount0 = liquidity * (sqrt_price_upper - sqrt_price_lower)
            / (sqrt_price_lower * sqrt_price_upper);
        return (amount0, Decimal::ZERO);
    }

    // Price above range: only token1
    if current_tick >= tick_upper {
        let amount1 = liquidity * (sqrt_price_upper - sqrt_price_lower);
        return (Decimal::ZERO, amount1);
    }

    // Price in range: both tokens
    let amount0 = liquidity * (sqrt_price_upper - sqrt_price)
        / (sqrt_price * sqrt_price_upper);
    let amount1 = liquidity * (sqrt_price - sqrt_price_lower);

    (amount0, amount1)
}

/// Calculate position value in terms of token1
///
/// Value = amount0 * price + amount1
///
/// This represents the total value of the position if we were to
/// convert all token0 to token1 at the current price
pub fn calculate_position_value(
    amount0: Decimal,
    amount1: Decimal,
    price: Decimal,
) -> Decimal {
    amount0 * price + amount1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_in_range() {
        assert!(is_in_range(100, 50, 150));
        assert!(is_in_range(50, 50, 150));
        assert!(!is_in_range(150, 50, 150));
        assert!(!is_in_range(30, 50, 150));
        assert!(!is_in_range(200, 50, 150));
    }

    #[test]
    fn test_distance_to_range_edge() {
        assert_eq!(distance_to_range_edge(100, 50, 150), 50);
        assert_eq!(distance_to_range_edge(75, 50, 150), 25);
        assert_eq!(distance_to_range_edge(125, 50, 150), 25);
        assert_eq!(distance_to_range_edge(30, 50, 150), 0);
        assert_eq!(distance_to_range_edge(200, 50, 150), 0);
    }

    #[test]
    fn test_tick_to_sqrt_price() {
        let sqrt_price_0 = tick_to_sqrt_price(0);
        assert!((sqrt_price_0 - Decimal::ONE).abs() < Decimal::from_str("0.0001").unwrap());

        // Positive tick should increase sqrt price
        let sqrt_price_100 = tick_to_sqrt_price(100);
        assert!(sqrt_price_100 > Decimal::ONE);

        // Negative tick should decrease sqrt price
        let sqrt_price_neg100 = tick_to_sqrt_price(-100);
        assert!(sqrt_price_neg100 < Decimal::ONE);

        // Verify sqrt_price^2 = price
        let tick = 1000;
        let sqrt_p = tick_to_sqrt_price(tick);
        let p = tick_to_price(tick);
        let calculated_price = sqrt_p * sqrt_p;
        assert!((calculated_price - p).abs() < Decimal::from_str("0.0001").unwrap());
    }

    #[test]
    fn test_tick_to_price() {
        let price_0 = tick_to_price(0);
        assert!((price_0 - Decimal::ONE).abs() < Decimal::from_str("0.0001").unwrap());

        // Positive tick should increase price
        let price_100 = tick_to_price(100);
        assert!(price_100 > Decimal::ONE);

        // Negative tick should decrease price
        let price_neg100 = tick_to_price(-100);
        assert!(price_neg100 < Decimal::ONE);

        // Test known values: price = 1.0001^tick
        // For tick = 1, price should be approximately 1.0001
        let price_1 = tick_to_price(1);
        assert!((price_1 - Decimal::from_str("1.0001").unwrap()).abs() < Decimal::from_str("0.000001").unwrap());
    }

    #[test]
    fn test_get_token_amounts_price_in_range() {
        let liquidity = Decimal::from(1000000);
        let tick_lower = -1000;
        let tick_upper = 1000;
        let current_tick = 0; // Price at 1.0

        let (amount0, amount1) = get_token_amounts_from_liquidity(
            liquidity,
            current_tick,
            tick_lower,
            tick_upper,
        );

        // When price is in middle of range, should have both tokens
        assert!(amount0 > Decimal::ZERO);
        assert!(amount1 > Decimal::ZERO);
    }

    #[test]
    fn test_get_token_amounts_price_below_range() {
        let liquidity = Decimal::from(1000000);
        let tick_lower = 1000;
        let tick_upper = 2000;
        let current_tick = 0; // Price below range

        let (amount0, amount1) = get_token_amounts_from_liquidity(
            liquidity,
            current_tick,
            tick_lower,
            tick_upper,
        );

        // When price is below range, should have only token0
        assert!(amount0 > Decimal::ZERO);
        assert_eq!(amount1, Decimal::ZERO);
    }

    #[test]
    fn test_get_token_amounts_price_above_range() {
        let liquidity = Decimal::from(1000000);
        let tick_lower = -2000;
        let tick_upper = -1000;
        let current_tick = 0; // Price above range

        let (amount0, amount1) = get_token_amounts_from_liquidity(
            liquidity,
            current_tick,
            tick_lower,
            tick_upper,
        );

        // When price is above range, should have only token1
        assert_eq!(amount0, Decimal::ZERO);
        assert!(amount1 > Decimal::ZERO);
    }

    #[test]
    fn test_get_token_amounts_zero_liquidity() {
        let (amount0, amount1) = get_token_amounts_from_liquidity(
            Decimal::ZERO,
            0,
            -1000,
            1000,
        );

        assert_eq!(amount0, Decimal::ZERO);
        assert_eq!(amount1, Decimal::ZERO);
    }

    #[test]
    fn test_calculate_position_value() {
        let amount0 = Decimal::from(100);
        let amount1 = Decimal::from(50);
        let price = Decimal::from(2);

        // Value = 100 * 2 + 50 = 250
        let value = calculate_position_value(amount0, amount1, price);
        assert_eq!(value, Decimal::from(250));
    }

    #[test]
    fn test_calculate_position_value_zero_price() {
        let amount0 = Decimal::from(100);
        let amount1 = Decimal::from(50);
        let price = Decimal::ZERO;

        // Value = 100 * 0 + 50 = 50
        let value = calculate_position_value(amount0, amount1, price);
        assert_eq!(value, Decimal::from(50));
    }

    #[test]
    fn test_price_to_tick_round_trip() {
        let original_tick = 1000;
        let price = tick_to_price(original_tick);
        let recovered_tick = price_to_tick(price);

        // Should be very close (within rounding)
        assert!((recovered_tick - original_tick).abs() <= 1);
    }
}
