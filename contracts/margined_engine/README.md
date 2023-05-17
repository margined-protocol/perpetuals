# Margined Protocol Engine

The margin engine is responsible for managing user positions and collateral. Allows users to take positions in all registered vAMMs.

---

## InstantiateMsg

The instantiation message takes the addresses of the insurance and fee contracts. It also takes the eligible collateral to be used and the margin ratios and liquidation fees.

```json
{
  "insurance_fund": "orai...",
  "fee_pool": "orai...",
  "eligible_collateral": "orai...",
  "initial_margin_ratio": "10000",
  "maintenance_margin_ratio": "10000",
  "liquidation_fee": "10000"
}
```

## ExecuteMsg

### `update_config`

Enables owner to update key contract parameters.

```json
{
    "update_config" {
        "owner": "orai...",
        "insurance_fund": "orai...",
        "fee_pool": "orai...",
        "eligible_collateral": "orai...",
        "initial_margin_ratio": "10000",
        "maintenance_margin_ratio": "10000",
        "partial_liquidation_ratio": "10000",
        "liquidation_fee": "1000",
    }
}
```

### `open_position`

Enables a user to open a position for a specific vAMM with leverage. Also allows order to be placed with slippage limits.

If side is buy (direction is 'add_to_amm') then open position (increase)

![Open Position Increase](/doc/diagrams/open-pos-increase.png)

If old position is larger then reduce position (decrease)

![Open Position Decrease](/doc/diagrams/open-pos-decrease.png)

Otherwise close position then swap out the entire position (reverse)

![Open Position Reverse](/doc/diagrams/open-pos-reverse.png)

```json
{
    "open_position" {
        "vamm": "orai...",
        "side": "buy",
        "quote_asset_amount": "10",
        "leverage": "1",
        "base_asset_limit": "0",
    }
}
```

### `close_position`

Enables a user to close a position they have for a specific vAMM including slippage limits.

![Close Position](/doc/diagrams/close-pos-partial.png)

If `partial_liquidation_ratio == 1` then close the whole position

![Close Whole Position](/doc/diagrams/close-pos-whole.png)

```json
{
    "close_position" {
        "vamm": "orai...",
        "quote_asset_limit": "0",
    }
}
```

### `liquidate`

Allows third parties to liquidate users positions when they are no longer sufficiently collateralised.

![Liquidate Position](/doc/diagrams/liq-pos-partial.png)

If `partial_liquidation_ratio == 0` then liquidate the whole position.

![Liquidate Whole Position](/doc/diagrams/liq-pos-whole.png)

```json
{
    "liquidate" {
        "vamm": "orai...",
        "trader": "orai...",
        "quote_asset_limit": "0",
    }
}
```

### `pay_funding`

Allows third parties to trigger funding payments to be processed for a specific vAMM.

![Pay Funding](/doc/diagrams/pay-funding.png)

```json
{
    "pay_funding" {
        "vamm": "orai...",
    }
}
```

### `deposit_margin`

Users can deposit additional margin to their positions to prevent them from becoming under-collateralised.

![Deposit Margin](/doc/diagrams/add-margin.png)

```json
{
    "deposit_margin" {
        "vamm": "orai...",
        "amount": "250000",
    }
}
```

### `withdraw_margin`

Users can withdraw excess collateral from their positions if they are over-collateralised

![Withdraw Margin](/doc/diagrams/remove-margin.png)

```json
{
    "withdraw_margin" {
        "vamm": "orai...",
        "amount": "250000",
    }
}

```

### `set_pause`

Enables owner to pause contracts in emergency situations

```json
{
    "set_pause" {
        "pause": true,
    }
}
```

## QueryMsg

### `config`

Returns the contracts configuration.

```json
{
    "config" {}
}
```

### `state`

Returns the state variables of the contract.

```json
{
    "state" {}
}
```

### `position`

Returns a user's position for a specific vAMM.

```json
{
    "position" {
        "vamm": "orai...",
        "trader": "orai...",
    }
}
```

### `all_positions`

Returns a user's positions for all vAMMs.

```json
{
    "all_positions" {
        "trader": "orai...",
    }
}
```

### `unrealized_pnl`

Returns the unrealized PnL (profit and loss) of a user for a specific vAMM using a specific calculation method.

```json
{
    "unrealized_pnl" {
        "vamm": "orai...",
        "trader": "orai...",
        "calc_option": "spot",
    }
}
```

### `cumulative_premium_fraction`

Returns the cumulative premium fraction of a vAMM.

```json
{
    "cumulative_premium_fraction" {
        "vamm": "orai...",
    }
}
```

### `margin_ratio`

Returns the margin ratio of a user for a vAMM.

```json
{
    "margin_ratio" {
        "vamm": "orai...",
        "trader": "orai...",
    }
}
```

### `free_collateral`

Returns the excess collateral a user has for a vAMM.

```json
{
    "free_collateral" {
        "vamm": "orai...",
        "trader": "orai...",
    }

}
```

### `balance_with_funding_payment`

Returns a user's margin balance across all vAMMs inclusive funding payments.

```json
{
    "balance_with_funding_payment" {
        "trader": "orai...",
    }
}
```

### `position_with_funding_payment`

Returns a user's margin balance inclusive funding payments for a specific vAMM.

```json
{
    "position_with_funding_payment" {
        "vamm": "orai...",
        "trader": "orai...",
    }
}
```
