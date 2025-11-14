# Impermanent Loss Calculation for Uniswap v3/v4 Concentrated Liquidity

This document explains the mathematical approach used to calculate impermanent loss (IL) for concentrated liquidity positions in Uniswap v3 and v4.

## Overview

Impermanent loss is the opportunity cost of providing liquidity compared to simply holding the tokens. For concentrated liquidity positions, IL calculations are more complex than full-range (Uniswap v2-style) positions because:

1. Liquidity is only active within a specific price range `[Pa, Pb]`
2. Token composition changes as price moves within the range
3. When price moves outside the range, the position becomes 100% one token

## Mathematical Foundation

### 1. Price and Tick Relationship

Uniswap v3/v4 uses ticks to represent discrete price points:

```
price = 1.0001^tick
sqrt_price = 1.0001^(tick/2)
P = (sqrt_price)^2
```

Each tick represents a 0.01% price movement (1 basis point).

### 2. Token Amounts from Liquidity

Given liquidity `L`, price range `[Pa, Pb]`, and current price `P`:

**When P is in range (Pa ≤ P < Pb):**
```
amount0 = L × (√Pb - √P) / (√P × √Pb)
amount1 = L × (√P - √Pa)
```

**When P < Pa (price below range):**
```
amount0 = L × (√Pb - √Pa) / (√Pa × √Pb)
amount1 = 0
```

**When P ≥ Pb (price above range):**
```
amount0 = 0
amount1 = L × (√Pb - √Pa)
```

These formulas come from equations 6.29 and 6.30 of the Uniswap v3 whitepaper.

### 3. Position Value Calculation

The total value of a position at price P (in terms of token1):

```
V = amount0 × P + amount1
```

Alternatively, in terms of token0:
```
V = amount0 + amount1 / P
```

### 4. Impermanent Loss Formula

IL compares the current position value to the hodl value (what you would have if you just held the tokens):

```
IL = (V_hodl - V_current) / V_hodl
```

Where:
- `V_hodl = x0 × P_current + y0` (initial token amounts valued at current price)
- `V_current = x_current × P_current + y_current` (current position valued at current price)

**IL > 0**: Loss (you would have been better off holding)
**IL < 0**: Gain (providing liquidity was better - rare, usually only with fees)

## Implementation Details

### Core Functions

#### `tick_to_sqrt_price(tick: i32) -> Decimal`

Converts a tick to sqrt price using the formula:
```
sqrt_price = 1.0001^(tick/2) = e^(tick/2 × ln(1.0001))
```

We use the exponential form for numerical stability.

#### `get_token_amounts_from_liquidity(L, current_tick, tick_lower, tick_upper) -> (Decimal, Decimal)`

Calculates the token amounts for a given liquidity and price, handling three cases:
1. Price in range: both tokens present
2. Price below range: only token0
3. Price above range: only token1

#### `calculate_impermanent_loss(position, initial_price, current_price) -> Decimal`

Main IL calculation that:
1. Converts prices to ticks
2. Calculates initial token amounts at initial price
3. Calculates current token amounts at current price
4. Computes hodl value vs current value
5. Returns IL as a positive number (loss) or zero

## Key Insights

### Concentrated Liquidity vs Full Range

**Concentrated liquidity positions can have HIGHER IL than full-range positions:**

- **Narrow ranges**: Higher capital efficiency but more IL for same price movement
- **Wide ranges**: Approaches full-range behavior with lower IL but less capital efficiency
- **Range positioning**: IL depends on where the price is relative to the range boundaries

### Edge Cases Handled

1. **Price moves outside range**: Position becomes 100% one token, no further IL accumulation in that direction
2. **Zero liquidity**: Returns IL of zero
3. **Invalid prices**: Returns IL of zero
4. **Initial price at boundary**: Handles positions initialized at range edges
5. **Large price movements**: Properly accounts for extreme price changes

### IL Behavior

- **Symmetric**: Price moving up 20% and down 20% (approximately) cause similar IL
- **Non-linear**: IL increases with price movement but not proportionally
- **Path-independent**: Only start and end prices matter, not the path taken
- **Range-dependent**: Narrower ranges → higher IL for same price movement

## Uniswap v4 Considerations

Uniswap v4 maintains the same concentrated liquidity mechanics as v3, so the core IL calculation remains unchanged. Key v4 features and their impact:

### No Impact on Core IL Math
- **Singleton architecture**: Internal optimization, doesn't affect position accounting
- **Flash accounting**: Gas optimization, doesn't change token amounts
- **Native ETH support**: Just a token type, same math applies

### Potential Edge Cases from Hooks
- **Custom fee structures**: Could affect net P&L but not pure IL
- **Dynamic fees**: Changes fee earnings, not IL calculation
- **Custom curves**: If hooks implement alternative AMM curves, this IL formula may not apply
- **Withdrawal fees**: Would affect realized value but not the IL calculation itself

**Note**: Our IL calculation assumes standard concentrated liquidity. If hooks modify the fundamental liquidity curve, additional analysis would be needed.

## Assumptions

1. **Constant Liquidity**: We assume the liquidity amount `L` doesn't change during the period. In practice, LPs can add/remove liquidity.

2. **No Fees Included**: This calculation computes *pure* impermanent loss without considering fee earnings. Total P&L = fees earned - IL - gas costs.

3. **Standard Liquidity Curve**: Assumes the standard Uniswap v3/v4 constant product curve within the range. Custom hook-based curves would require different formulas.

4. **Price Accuracy**: Tick-to-price conversions use `rust_decimal` for precision, but extremely large ticks are capped to prevent overflow.

5. **Token Decimals**: The calculation works in normalized decimal units. Actual token amounts would need to be adjusted for token decimals.

6. **No Rebalancing**: Assumes the position is not actively rebalanced. Active management strategies would have different P&L profiles.

## Accuracy and Precision

### Numerical Stability
- Uses `rust_decimal` with 28-29 significant digits
- Exponential calculations via `exp(x × ln(1.0001))` for stability
- Caps extreme tick values to prevent overflow

### Sources of Error
1. **Tick-to-price conversion**: Small rounding in exponential calculation
2. **Decimal arithmetic**: Standard floating-point considerations
3. **Price-to-tick round trip**: May be off by ±1 tick due to rounding

### Validation
All formulas are validated against:
- Known mathematical properties (symmetry, bounds)
- Edge cases (zero liquidity, price at boundaries, extreme moves)
- Uniswap v3 whitepaper equations 6.29 and 6.30

## Usage Example

```rust
use stillwater_analytics::calculate_impermanent_loss;
use stillwater_models::Position;
use rust_decimal::Decimal;

let position = Position {
    liquidity: U256::from(1000000),
    tick_lower: -1000,  // ~0.905 price
    tick_upper: 1000,   // ~1.105 price
    // ... other fields
};

let initial_price = Decimal::from_str("1.0").unwrap();
let current_price = Decimal::from_str("1.1").unwrap(); // 10% increase

let il = calculate_impermanent_loss(&position, initial_price, current_price);
println!("Impermanent loss: {:.2}%", il * Decimal::from(100));
```

## References

1. **Uniswap v3 Whitepaper**: https://uniswap.org/whitepaper-v3.pdf
   - Equations 6.29 and 6.30 for token amounts from liquidity
   - Section 6.2 on price and liquidity

2. **Uniswap v4 Whitepaper**: https://uniswap.org/whitepaper-v4.pdf
   - Confirms concentrated liquidity mechanics unchanged
   - Describes hooks and their potential impacts

3. **Technical Analysis**: "Liquidity Math in Uniswap v3" by Atis Elsts
   - Detailed derivations and additional formulas
   - Available at: https://atiselsts.github.io/pdfs/uniswap-v3-liquidity-math.pdf

## Future Enhancements

Potential improvements to consider:

1. **Time-weighted IL**: Calculate IL over multiple time periods with snapshots
2. **Fee-adjusted IL**: Include actual fee earnings in P&L calculation
3. **Gas cost tracking**: Full P&L including transaction costs
4. **Multiple positions**: Aggregate IL across a portfolio of positions
5. **Hook-aware calculations**: Special handling for positions with custom hooks
6. **Real-time tracking**: Stream IL calculations as prices update
7. **Historical analysis**: Compare IL to historical volatility and fee earnings
