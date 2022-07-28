# Margined Margin Engine

The margin engine is responsible for managing user positions and collateral. Allows users to take positions in all registered vAMMs.

---

## InstantiateMsg

The instantiation message takes the decimals to be used by the contract, the addresses of the insurance and fee contracts. It also takes the eligible collateral to be used and the margin ratios and liquidation fees.

```json
{
    "decimals": 6,
    "insurance_fund": "juno...",
    "fee_pool": "juno...",
    "eligible_collateral": "juno...",
    "initial_margin_ratio": "10000",
    "maintenance_margin_ratio": "10000",
    "liquidation_fee": "10000",
}
```

## ExecuteMsg

### `update_config`

Enables owner to update key contract parameters.

```json
{
    "update_config" {
        "owner": "juno...",
        "insurance_fund": "juno...",
        "fee_pool": "juno...",
        "eligible_collateral": "juno...",
        "decimals": "6",
        "initial_margin_ratio": "10000",
        "maintenance_margin_ratio": "10000",
        "partial_liquidation_margin_ratio": "10000",
        "liquidation_fee": "1000",
    }
}
```

### `open_position`

Enables a user to open a position for a specific vAMM with leverage. Also allows order to be placed with slippage limits.

```json
{
    "open_position" {
        "vamm": "juno...",
        "side": "buy",
        "quote_asset_amount": "10",
        "leverage": "1",
        "base_asset_limit": "0",
    }
}
```
    
### `close_position`

Enables a user to close a position they have for a specific vAMM including slippage limits.

```json
{
    "close_position" {
        "vamm": "juno...",
        "quote_asset_limit": "0",
    }
}
```

### `liquidate`

Allows third parties to liquidate users positions when they are no longer sufficiently collateralised.

```json
{
    "liquidate" {
        "vamm": "juno...",
        "trader": "juno...",
        "quote_asset_limit": "0",
    }
}
```

### `pay_funding`

Allows third parties to trigger funding payments to be processed for a specific vAMM.

```json
{
    "pay_funding" {
        "vamm": "juno...",
    }
}
```
    

### `deposit_margin`

Users can deposit additional margin to their positions to prevent them from becoming under-collateralised.

```json
{
    "deposit_margin" {
        "vamm": "juno...",
        "amount": "250000",
    }
}
```

### `withdraw_margin`

Users can withdraw excess collateral from their positions if they are over-collateralised

```json
{
    "withdraw_margin" {
        "vamm": "juno...",
        "amount": "250000",
    }
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
        "vamm": "juno...",
        "trader": "juno...",
    }   
}
```

### `all_positions`

Returns a user's positions for all vAMMs.

```json
{
    "all_positions" {
        "trader": "juno...",
    }
}
```    

### `unrealized_pnl`

Returns the unrealized PnL (profit and loss) of a user for a specific vAMM using a specific calculation method.

```json
{
    "unrealized_pnl" {
        "vamm": "juno...",
        "trader": "juno...",
        "calc_option": "spot",
    }
}
```
    

### `cumulative_premium_fraction`

Returns the cumulative premium fraction of a vAMM.

```json
{
    "cumulative_premium_fraction" {
        "vamm": "juno...",
    }
}
```

### `margin_ratio`

Returns the margin ratio of a user for a vAMM.

```json
{
    "margin_ratio" {
        "vamm": "juno...",
        "trader": "juno...",
    }    
}
```

### `free_collateral`

Returns the excess collateral a user has for a vAMM.

```json
{
    "free_collateral" {
        "vamm": "juno...",
        "trader": "juno...",
    }
  
}
```

### `balance_with_funding_payment`

Returns a user's margin balance across all vAMMs inclusive funding payments.

```json
{
    "balance_with_funding_payment" {
        "trader": "juno...",
    }    
}
```

### `position_with_funding_payment`

Returns a user's margin balance inclusive funding payments for a specific vAMM.

```json
{
    "position_with_funding_payment" {
        "vamm": "juno...",
        "trader": "juno...",
    }
}
```

