# Margined Protocol Price Feed

**NOTE:** In the current state the price feed is entirely centralised around the contract deployer just while the other logic is developed.

Price feed integrates against the [TeFi oracle hub](https://github.com/terra-money/tefi-oracle-contracts). Additionally, the price feed performs other logic, e.g. TWAP, of data retrieved from the data oracles for use throughout the protocol.