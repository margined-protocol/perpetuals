# Margined Protocol Price Feed

**NOTE:** In the current state the price feed is entirely centralised around the contract deployer just while the other logic is developed.

Price feed was meant to integrate against the [TeFi oracle hub](https://github.com/terra-money/tefi-oracle-contracts), but obvs not gonna fly anymore. Additionally, the price feed performs other logic, e.g. TWAP, of data retrieved from the data oracles for use throughout the protocol.

---

## InstantiateMsg

The instantiation message takes the decimals to be used by the contract, the addresses of the insurance and fee contracts. It also takes the eligible collateral to be used and the margin ratios and liquidation fees.

```json
{
    
}
```

## ExecuteMsg

### `example_execute`

```json
{
    
}
```

## QueryMsg

### `example_query`

```json
{
    
}
```