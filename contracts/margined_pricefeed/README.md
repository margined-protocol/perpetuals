# Margined Protocol Price Feed

**NOTE:** In the current state the price feed is entirely centralised around the contract deployer just while the other logic is developed.

Price feed was meant to integrate against the [TeFi oracle hub](https://github.com/terra-money/tefi-oracle-contracts), but obvs not gonna fly anymore. Additionally, the price feed performs other logic, e.g. TWAP, of data retrieved from the data oracles for use throughout the protocol.

---

## InstantiateMsg

The instantiation message takes the oracle hub contract, that would be used in a production version.
```json
{
    "oracle_hub_contract": "juno..."
}
```

## ExecuteMsg

### `update_config`

```json
{
    "update_config": {
        "owner": "juno..."
    }
}
```

## QueryMsg

### `config`

Returns contract parameters.

```json
{
    "config": {}
}
```

### `get_price`

Returns latest price submitted to the contract.

```json
{
    "get_price": {
        "key": "BTC",
    }
}
```

### `get_previous_price`

Returns a price submitted in a previous round.

```json
{
    "get_previous_price": {
        "key": "BTC",
        "num_round_back": 9,
    }
}
```

### `get_twap_price`

Returns a twap of the prices submitted to the contract.

```json
{
    "get_twap_price": {
        "key": "BTC",
        "interval": 900,
    }
}
```