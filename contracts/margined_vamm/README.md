# Margined Protocol _virtual_ Automated Market Maker

The vAMM is the contract that enables user's to take perpetual positions through its "virtual" automated market maker.

The vAMM also integrates with the price feed contracts for access to market data, such as Orderbook or AMM on Oraichain and Kucoin.

---

## InstantiateMsg

The instantiation message takes the decimals to be used by the contract, the addresses of the pricefeed and margin engine contracts. It also takes the definition of the product pair to be traded on the vAMM including the initial liquidity.

```json
{
  "decimals": 6,
  "pricefeed": "orai...",
  "margin_engine": "orai...",
  "quote_asset": "USDT",
  "base_asset": "ORAI",
  "quote_asset_reserve": "12000",
  "base_asset_reserve": "4000",
  "funding_period": "3600",
  "toll_ratio": "5000",
  "spread_ratio": "5000",
  "fluctuation_limit_ratio": "5000"
}
```

## ExecuteMsg

### `update_config`

Enables owner to update key contract parameters.

```json
{
    "update_config" {
        "base_asset_holding_cap": "10000000",
        "open_interest_notional_cap": "10000000",
        "toll_ratio": "10000",
        "spread_ratio": "10000",
        "fluctuation_limit_ratio": "10000",
        "margin_engine": "orai...",
        "pricefeed": "orai...",
        "spot_price_twap_interval": 6,
    }
}
```

### `update_owner`

Transfers the contract owner.

```json
{
  "update_owner": {
    "owner": "orai..."
  }
}
```

### `swap_input`

Allows the margin engine to swap quote asset into the vAMM.

```json
{
    "swap_input" {
        "direction": "add_to_amm",
        "quote_asset_amount": "10000000",
        "base_asset_limit": "10000000",
        "can_go_over_fluctuation": false,
    }
}
```

### `swap_output`

Allows the margin engine to swap base asset into the vAMM.

```json
{
    "swap_output" {
        "direction": "remove_from_amm",
        "quote_asset_amount": "10000000",
        "base_asset_limit": "10000000",
    }
}
```

### `settle_funding`

Calculates the funding payments due.

```json
{
    "settle_funding" {}
}
```

### `set_open`

Allows owner to open the vAMM enable positions to be taken.

```json
{
    "set_open" {
        "open": true
    }
}
```

## QueryMsg

### `config`

Returns contract configuration.

```json
{
  "config": {}
}
```

### `state`

Returns contract state, including liquidity etc.

```json
{
  "state": {}
}
```

### `input_price`

Returns the average price for a trade of a given size.

```json
{
  "input_price": {
    "direction": "add_to_amm",
    "amount": "10000000"
  }
}
```

### `output_price`

Returns the average price for a trade of a given size.

```json
{
  "output_price": {
    "direction": "add_to_amm",
    "amount": "10000000"
  }
}
```

### `input_amount`

Returns the amount for a trade of input with a given size.

```json
{
  "input_amount": {
    "direction": "add_to_amm",
    "amount": "10000000"
  }
}
```

### `output_amount`

Returns the amount for a trade of output with a given size.

```json
{
  "output_amount": {
    "direction": "add_to_amm",
    "amount": "10000000"
  }
}
```

### `input_twap`

Returns input twap price of the vAMM, using the reserve snapshots, default 15 minutes interval.

```json
{
  "input_twap": {
    "direction": "add_to_amm",
    "amount": "10000000"
  }
}
```

### `output_twap`

Returns output twap price of the vAMM, using the reserve snapshots, default 15 minutes interval.

```json
{
  "output_twap": {
    "direction": "add_to_amm",
    "amount": "10000000"
  }
}
```

### `spot_price`

Returns spot price of the vAMM.

```json
{
  "spot_price": {}
}
```

### `twap_price`

Return twap price of the vAMM, using the reserve snapshots.

```json
{
  "twap_price": {
    "interval": 900
  }
}
```

### `calc_fee`

Returns the total (i.e. toll + spread) fees for an amount.

```json
{
  "calc_fee": {
    "quote_asset_amount": "10000000"
  }
}
```

### `is_over_spread_limit`

Returns bool to show is spread limit has been exceeded.

```json
{
  "is_over_spread_limit": {}
}
```
