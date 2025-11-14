# Stillwater Analytics

Analytics and calculations for liquidity provider positions on Uniswap v4.

## Overview

This crate provides accurate P&L tracking for concentrated liquidity positions, including:

- **Impermanent Loss (IL)** calculations based on Uniswap v3/v4 formulas
- **Fee earnings** estimation from swap volumes
- **Position health** monitoring and risk assessment
- **Utility functions** for tick/price conversions and range analysis

## Features

### Impermanent Loss Calculation

Accurate IL calculation for concentrated liquidity positions that accounts for:
- Price movements within custom ranges `[Pa, Pb]`
- Position-specific liquidity amounts
- Edge cases (price outside range, boundary conditions)
- Both price increases and decreases

See [IMPERMANENT_LOSS.md](./IMPERMANENT_LOSS.md) for detailed mathematical derivations and formulas.

### Position P&L

Complete P&L calculation combining:
```
Net P&L = Fees Earned - Impermanent Loss - Gas Costs
```

### Health Monitoring

Position health assessment based on:
- Current price relative to range
- Distance to range boundaries
- Net P&L (positive/negative)

Health statuses:
- **Healthy**: In range with positive P&L
- **Warning**: Near range edge or small negative P&L
- **Critical**: Out of range or significant negative P&L

### Utility Functions

- `tick_to_price(tick)` - Convert tick to price
- `tick_to_sqrt_price(tick)` - Convert tick to sqrt price (for internal calculations)
- `price_to_tick(price)` - Convert price to tick
- `get_token_amounts_from_liquidity(L, tick, tick_lower, tick_upper)` - Calculate token amounts
- `calculate_position_value(amount0, amount1, price)` - Calculate position value in token1 terms
- `is_in_range(tick, tick_lower, tick_upper)` - Check if position is active
- `distance_to_range_edge(tick, tick_lower, tick_upper)` - Ticks to nearest boundary
- `range_width_percent(tick_lower, tick_upper)` - Range width as percentage

## Usage

```rust
use stillwater_analytics::{
    calculate_impermanent_loss,
    calculate_position_pnl,
    get_position_health,
    tick_to_price,
};
use stillwater_models::Position;
use rust_decimal::Decimal;

// Calculate IL
let position = /* your position */;
let initial_price = Decimal::from_str("1.0")?;
let current_price = Decimal::from_str("1.1")?;

let il = calculate_impermanent_loss(&position, initial_price, current_price);
println!("Impermanent loss: {:.2}%", il * Decimal::from(100));

// Calculate complete P&L
let swaps = /* fetch swaps for this position */;
let gas_spent = Decimal::from_str("0.05")?;

let pnl = calculate_position_pnl(
    &position,
    &swaps,
    initial_price,
    current_price,
    gas_spent,
);

println!("Fees earned: {}", pnl.fees_earned);
println!("Impermanent loss: {}", pnl.impermanent_loss);
println!("Net P&L: {}", pnl.net_pnl);

// Check position health
let current_tick = /* get current pool tick */;
let health = get_position_health(&position, current_tick, pnl.net_pnl);
println!("Position health: {:?}", health);
```

## Mathematical Accuracy

The implementation follows the Uniswap v3/v4 whitepaper formulas exactly:

- Uses `rust_decimal` for high precision (28-29 significant digits)
- Implements equations 6.29 and 6.30 from Uniswap v3 whitepaper
- Handles edge cases (price outside range, zero liquidity, etc.)
- Comprehensive unit tests covering various scenarios

### Key Formulas

**Token amounts from liquidity** (when price P is in range [Pa, Pb]):
```
amount0 = L × (√Pb - √P) / (√P × √Pb)
amount1 = L × (√P - √Pa)
```

**Impermanent loss**:
```
IL = (V_hodl - V_current) / V_hodl

where:
  V_hodl = x0 × P_current + y0
  V_current = x_current × P_current + y_current
```

## Testing

Run the test suite:
```bash
cargo test -p stillwater-analytics
```

The tests cover:
- IL calculation with various price movements (up, down, large, small)
- Edge cases (price outside range, zero liquidity, boundary conditions)
- Helper functions (tick conversions, token amount calculations)
- Position health assessment
- Numerical accuracy and stability

## Assumptions

1. **Constant liquidity**: Position's liquidity `L` doesn't change
2. **No fees in IL**: Pure IL calculation, fees are tracked separately
3. **Standard curve**: Assumes standard Uniswap v3/v4 constant product formula
4. **No custom hooks**: Hooks with custom curves may require different calculations

See [IMPERMANENT_LOSS.md](./IMPERMANENT_LOSS.md) for complete assumptions and limitations.

## Integration with Stillwater

This crate is used by:
- **stillwater-api**: REST API endpoints for position analytics
- **stillwater-db**: Storing calculated P&L in the database
- **stillwater-indexer**: Real-time position monitoring

## References

- [Uniswap v3 Whitepaper](https://uniswap.org/whitepaper-v3.pdf)
- [Uniswap v4 Whitepaper](https://uniswap.org/whitepaper-v4.pdf)
- [Liquidity Math in Uniswap v3](https://atiselsts.github.io/pdfs/uniswap-v3-liquidity-math.pdf) by Atis Elsts

## License

See [LICENSE](../../LICENSE) file in the repository root.
