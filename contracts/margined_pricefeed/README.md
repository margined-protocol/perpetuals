# Margined Protocol Price Feed

**NOTE:** In the current state the price feed is entirely centralized around the contract deployer just while the other logic is developed.

Price feed was meant to integrate against the [TeFi oracle hub](https://github.com/oraichain/oracle-hub), but obvs not gonna fly anymore. Additionally, the price feed performs other logic, e.g. TWAP, of data retrieved from the data oracles for use throughout the protocol.
In next version it will utilizing Orai oracle pricefeed.

---

## InstantiateMsg

The instantiation message takes the oracle hub contract, that would be used in a production version.

```json
{
  "oracle_hub_contract": "orai..."
}
```

## ExecuteMsg

### `update_owner`

Transfers the contract owner.

```json
{
  "update_owner": {
    "owner": "orai..."
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
    "key": "ORAI"
  }
}
```

### `get_previous_price`

Returns a price submitted in a previous round.

```json
{
  "get_previous_price": {
    "key": "ORAI",
    "num_round_back": 9
  }
}
```

### `get_twap_price`

Returns a twap of the prices submitted to the contract.

```json
{
  "get_twap_price": {
    "key": "ORAI",
    "interval": 900
  }
}
```
