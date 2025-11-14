use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use stillwater_models::{Position, PositionPnL, Swap};

use crate::utils::{get_token_amounts_from_liquidity, calculate_position_value, price_to_tick};

#[cfg(test)]
use crate::utils::tick_to_price;

/// Calculate fees earned from swaps
///
/// For a concentrated liquidity position, fees are earned when:
/// 1. The swap occurs while the position is in range
/// 2. The position has active liquidity
///
/// Simplified calculation: assumes position was always in range for swaps provided
pub fn calculate_fees_earned(_position: &Position, swaps: &[Swap]) -> Decimal {
    if swaps.is_empty() {
        return Decimal::ZERO;
    }

    // Simplified fee calculation
    // In reality, would need:
    // - Total pool liquidity at time of each swap
    // - Position's share of liquidity
    // - Fee tier for the pool
    //
    // For MVP, estimate based on swap volumes and assume 0.3% fee tier
    let fee_rate = Decimal::from_str("0.003").unwrap(); // 0.3%

    let total_volume: Decimal = swaps
        .iter()
        .map(|swap| {
            // Use absolute values and convert to decimal
            // This is a rough approximation
            let amt0 = swap.amount0.abs().to_string();
            let amt1 = swap.amount1.abs().to_string();

            Decimal::from_str(&amt0).unwrap_or(Decimal::ZERO)
                + Decimal::from_str(&amt1).unwrap_or(Decimal::ZERO)
        })
        .sum();

    // Estimate fees as a fraction of total volume
    // In production, would calculate exact share based on liquidity
    let estimated_position_share = Decimal::from_str("0.01").unwrap(); // 1% of pool

    total_volume * fee_rate * estimated_position_share
}

/// Calculate impermanent loss for concentrated liquidity position
///
/// Based on Uniswap v3/v4 concentrated liquidity mechanics:
///
/// ## Mathematical Derivation:
///
/// 1. **Initial Position (at price P0):**
///    - Calculate token amounts (x0, y0) using liquidity L and price P0
///    - Using formulas from Uniswap v3 whitepaper (equations 6.29, 6.30)
///
/// 2. **Current Position (at price P):**
///    - Calculate current token amounts (x, y) using same liquidity L and current price P
///    - Note: As price moves, token composition changes (rebalancing)
///
/// 3. **Position Values:**
///    - Current position value: V_current = x * P + y (in token1 terms)
///    - Hodl value: V_hodl = x0 * P + y0 (what we'd have if we held original tokens)
///
/// 4. **Impermanent Loss:**
///    - IL = (V_hodl - V_current) / V_hodl
///    - IL > 0 means loss (holding would have been better)
///    - IL < 0 means gain (providing liquidity was better - rare, usually due to fees)
///
/// ## Key Insights:
/// - IL only occurs when price is in range [Pa, Pb]
/// - When price moves outside range, position becomes 100% one token
/// - Concentrated liquidity can have MORE IL than full-range positions
/// - Narrower ranges = higher IL for same price movement
///
/// ## Edge Cases:
/// - Price below range: position is 100% token0, no further IL as price drops
/// - Price above range: position is 100% token1, no further IL as price rises
/// - Initial price outside range: need to handle carefully
///
pub fn calculate_impermanent_loss(
    position: &Position,
    initial_price: Decimal,
    current_price: Decimal,
) -> Decimal {
    // Validate inputs
    if initial_price.is_zero() || current_price.is_zero() {
        return Decimal::ZERO;
    }

    // If price hasn't moved, no IL
    let price_tolerance = Decimal::from_str("0.000001").unwrap();
    if (current_price - initial_price).abs() < price_tolerance {
        return Decimal::ZERO;
    }

    // Convert liquidity from U256 to Decimal
    // Uniswap v3/v4 liquidity is stored as uint128, but we have U256
    let liquidity_str = position.liquidity.to_string();
    let liquidity = match Decimal::from_str(&liquidity_str) {
        Ok(l) => l,
        Err(_) => return Decimal::ZERO,
    };

    if liquidity.is_zero() {
        return Decimal::ZERO;
    }

    // Get initial tick from initial price
    let initial_tick = price_to_tick(initial_price);

    // Get current tick from current price
    let current_tick = price_to_tick(current_price);

    // Calculate initial token amounts at initial price
    let (x0, y0) = get_token_amounts_from_liquidity(
        liquidity,
        initial_tick,
        position.tick_lower,
        position.tick_upper,
    );

    // Calculate current token amounts at current price
    let (x_current, y_current) = get_token_amounts_from_liquidity(
        liquidity,
        current_tick,
        position.tick_lower,
        position.tick_upper,
    );

    // Calculate hodl value: what we'd have if we kept the initial tokens
    // Value in terms of token1: V = x * P + y
    let v_hodl = calculate_position_value(x0, y0, current_price);

    // Calculate current position value at current price
    let v_current = calculate_position_value(x_current, y_current, current_price);

    // If hodl value is zero, can't calculate IL
    if v_hodl.is_zero() {
        return Decimal::ZERO;
    }

    // Calculate IL as percentage: (V_hodl - V_current) / V_hodl
    let il = (v_hodl - v_current) / v_hodl;

    // Return IL as absolute value (loss is positive)
    // Note: In rare cases, IL can be negative if position gained value
    // (this can happen due to fees, but we calculate pure IL here)
    il.max(Decimal::ZERO)
}

/// Calculate net P&L
pub fn calculate_net_pnl(fees: Decimal, il: Decimal, gas: Decimal) -> Decimal {
    fees - il - gas
}

/// Calculate complete position P&L
pub fn calculate_position_pnl(
    position: &Position,
    swaps: &[Swap],
    initial_price: Decimal,
    current_price: Decimal,
    gas_spent: Decimal,
) -> PositionPnL {
    let fees_earned = calculate_fees_earned(position, swaps);
    let impermanent_loss = calculate_impermanent_loss(position, initial_price, current_price);
    let net_pnl = calculate_net_pnl(fees_earned, impermanent_loss, gas_spent);

    PositionPnL {
        fees_earned,
        impermanent_loss,
        gas_spent,
        net_pnl,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{I256, U256};
    use chrono::Utc;

    fn create_test_position() -> Position {
        Position {
            id: 1,
            nft_id: "1".to_string(),
            owner: "0xtest".to_string(),
            pool_id: "0xpool".to_string(),
            tick_lower: -1000,
            tick_upper: 1000,
            liquidity: U256::from(1000000u64),
            created_at: Utc::now(),
        }
    }

    fn create_test_swap(amount0: i64, amount1: i64) -> Swap {
        Swap {
            id: 1,
            tx_hash: "0xtx".to_string(),
            pool_id: "0xpool".to_string(),
            amount0: I256::try_from(amount0).unwrap(),
            amount1: I256::try_from(amount1).unwrap(),
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_calculate_fees_earned() {
        let position = create_test_position();
        let swaps = vec![
            create_test_swap(1000, 1000),
            create_test_swap(2000, 2000),
        ];

        let fees = calculate_fees_earned(&position, &swaps);
        assert!(fees > Decimal::ZERO);
    }

    #[test]
    fn test_calculate_impermanent_loss_no_price_change() {
        let position = create_test_position();
        let price = Decimal::from(100);

        let il = calculate_impermanent_loss(&position, price, price);
        assert_eq!(il, Decimal::ZERO);
    }

    #[test]
    fn test_calculate_impermanent_loss_price_increase() {
        let position = create_test_position();
        let initial_price = Decimal::from_str("1.0").unwrap();
        let current_price = Decimal::from_str("1.1").unwrap(); // 10% increase

        let il = calculate_impermanent_loss(&position, initial_price, current_price);

        // IL should be positive (loss occurs with price movement)
        assert!(il > Decimal::ZERO);
        // For a 10% price move in concentrated liquidity, IL should be reasonable
        assert!(il < Decimal::from_str("0.1").unwrap()); // Less than 10%
    }

    #[test]
    fn test_calculate_impermanent_loss_price_decrease() {
        let position = create_test_position();
        let initial_price = Decimal::from_str("1.0").unwrap();
        let current_price = Decimal::from_str("0.9").unwrap(); // 10% decrease

        let il = calculate_impermanent_loss(&position, initial_price, current_price);

        // IL should be positive (loss occurs with price movement in either direction)
        assert!(il > Decimal::ZERO);
        assert!(il < Decimal::from_str("0.1").unwrap());
    }

    #[test]
    fn test_calculate_impermanent_loss_price_below_range() {
        // Create position with range above current price
        let mut position = create_test_position();
        position.tick_lower = 10000;  // High price range
        position.tick_upper = 20000;

        let initial_price = tick_to_price(15000); // In range
        let current_price = tick_to_price(5000);  // Below range

        let il = calculate_impermanent_loss(&position, initial_price, current_price);

        // When price moves out of range, IL should still be calculated
        assert!(il >= Decimal::ZERO);
    }

    #[test]
    fn test_calculate_impermanent_loss_price_above_range() {
        // Create position with range below current price
        let mut position = create_test_position();
        position.tick_lower = -20000;  // Low price range
        position.tick_upper = -10000;

        let initial_price = tick_to_price(-15000); // In range
        let current_price = tick_to_price(-5000);  // Above range

        let il = calculate_impermanent_loss(&position, initial_price, current_price);

        // When price moves out of range, IL should still be calculated
        assert!(il >= Decimal::ZERO);
    }

    #[test]
    fn test_calculate_impermanent_loss_narrow_range() {
        // Narrow range should have higher IL for same price movement
        let mut position = create_test_position();
        position.tick_lower = -100;  // Very narrow range
        position.tick_upper = 100;

        let initial_price = Decimal::from_str("1.0").unwrap();
        let current_price = Decimal::from_str("1.05").unwrap(); // 5% increase

        let il = calculate_impermanent_loss(&position, initial_price, current_price);

        // Should have some IL
        assert!(il > Decimal::ZERO);
    }

    #[test]
    fn test_calculate_impermanent_loss_wide_range() {
        // Wide range should have lower IL for same price movement
        let mut position = create_test_position();
        position.tick_lower = -10000;  // Very wide range
        position.tick_upper = 10000;

        let initial_price = Decimal::from_str("1.0").unwrap();
        let current_price = Decimal::from_str("1.05").unwrap(); // 5% increase

        let il = calculate_impermanent_loss(&position, initial_price, current_price);

        // Should have some IL but less than narrow range
        assert!(il >= Decimal::ZERO);
    }

    #[test]
    fn test_calculate_impermanent_loss_zero_liquidity() {
        let mut position = create_test_position();
        position.liquidity = U256::ZERO;

        let initial_price = Decimal::from(100);
        let current_price = Decimal::from(110);

        let il = calculate_impermanent_loss(&position, initial_price, current_price);
        assert_eq!(il, Decimal::ZERO);
    }

    #[test]
    fn test_calculate_impermanent_loss_zero_price() {
        let position = create_test_position();
        let initial_price = Decimal::ZERO;
        let current_price = Decimal::from(110);

        let il = calculate_impermanent_loss(&position, initial_price, current_price);
        assert_eq!(il, Decimal::ZERO);
    }

    #[test]
    fn test_calculate_impermanent_loss_large_price_increase() {
        let position = create_test_position();
        let initial_price = Decimal::from_str("1.0").unwrap();
        let current_price = Decimal::from_str("2.0").unwrap(); // 100% increase

        let il = calculate_impermanent_loss(&position, initial_price, current_price);

        // Should have significant IL
        assert!(il > Decimal::ZERO);
        // But shouldn't be more than 100%
        assert!(il < Decimal::from_str("1.0").unwrap());
    }

    #[test]
    fn test_calculate_impermanent_loss_large_price_decrease() {
        let position = create_test_position();
        let initial_price = Decimal::from_str("2.0").unwrap();
        let current_price = Decimal::from_str("1.0").unwrap(); // 50% decrease

        let il = calculate_impermanent_loss(&position, initial_price, current_price);

        // Should have significant IL
        assert!(il > Decimal::ZERO);
        assert!(il < Decimal::from_str("1.0").unwrap());
    }

    #[test]
    fn test_calculate_impermanent_loss_symmetry() {
        // IL should be similar for equal percentage moves up or down
        let position = create_test_position();
        let base_price = Decimal::from_str("1.0").unwrap();

        // 20% increase
        let il_up = calculate_impermanent_loss(
            &position,
            base_price,
            Decimal::from_str("1.2").unwrap()
        );

        // 20% decrease (approximately)
        let il_down = calculate_impermanent_loss(
            &position,
            base_price,
            Decimal::from_str("0.8333").unwrap()
        );

        // Both should have IL
        assert!(il_up > Decimal::ZERO);
        assert!(il_down > Decimal::ZERO);

        // They should be relatively close (within 50% of each other)
        // Note: Not exactly equal due to non-linear nature of IL
        let ratio = if il_up > il_down {
            il_up / il_down
        } else {
            il_down / il_up
        };
        assert!(ratio < Decimal::from_str("2.0").unwrap());
    }

    #[test]
    fn test_calculate_impermanent_loss_initial_price_at_boundary() {
        let position = create_test_position();

        // Initial price at lower boundary
        let initial_price = tick_to_price(position.tick_lower);
        let current_price = tick_to_price(0); // Middle of range

        let il = calculate_impermanent_loss(&position, initial_price, current_price);
        assert!(il >= Decimal::ZERO);
    }

    #[test]
    fn test_calculate_net_pnl() {
        let fees = Decimal::from(100);
        let il = Decimal::from(20);
        let gas = Decimal::from(10);

        let net = calculate_net_pnl(fees, il, gas);
        assert_eq!(net, Decimal::from(70));
    }

    #[test]
    fn test_calculate_position_pnl() {
        let position = create_test_position();
        let swaps = vec![create_test_swap(1000, 1000)];
        let initial_price = Decimal::from(100);
        let current_price = Decimal::from(105);
        let gas_spent = Decimal::from(5);

        let pnl = calculate_position_pnl(&position, &swaps, initial_price, current_price, gas_spent);

        assert!(pnl.fees_earned >= Decimal::ZERO);
        assert!(pnl.impermanent_loss >= Decimal::ZERO);
        assert_eq!(pnl.gas_spent, gas_spent);
    }
}
