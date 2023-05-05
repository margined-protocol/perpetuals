# Margined Protocol Architecture

This is the internal architecture document for Margined Protocol

[Design Doc](./v1-design.md)

## Component Diagram:

![Component Diagram](./diagrams/component-diagram.png)

1. Users are able to interact with all public functions, including open/close positions and liquidations
2. Governance is able to perform config updates and contract migrations for both the insurance fund and margin engine contracts
3. A portion of trading fees are deposited into the insurance fund
4. Funds may be withdrawn to cover a shortfall
5. In an emergency governance can trigger shutdown of all vAMMs through the insurance fund
6. The engine deposits a portion of the fees into the fee pool
7. Revenue generated is redistributed to token holders
8. The vAMM manages protocol liquidity during all input and output swaps triggered by the margin engine
9. Index price is retrieved from a pricefeed during the settlement of funding payments

## Component Descriptions

### Governance

Protocol governance is able to configure and migrate contracts.

### Users

As a decentralised protocol users are free to interact with the protocol as they wish. Users have two roles:

- Traders
- Liquidators

Traders are users who take perpetual positions within the protocol. Liquidators are users who monitor the margin levels of users and perform liquidations appropriately.

### Margin Engine

The margin engine manages the positions of protocol users and is permissoned to perform actions with the vAMMs on behalf of users. Traders use the margin engine contract to open and close positions. Additionally, traders are able to deposit and remove margin from their margin account within the engine as appropriate.

The margin engine holds all collateral of the traders, however should a shortfall occur the margin engine is able to withdraw funds from the insurance fund in order to cover a shortfall.

During the settlement of funding payments the margin engine applies the funding payment to the margin account of users.

As part of the fee collection the margin engine pays a portion of the trading fees into the insurance fund and fee pool.

### vAMM

The vAMM acts as the mechanism for price discovery. The contract records the reserves of base and quote assets, and enables swapping input and output - returning a price to the engine. The vAMM also returns the size of the position that a user purchases, which is the amount of base asset.

The vAMM also uses the pricefeed contract to find the oracle price during settlement of funding payments.

### Insurance Fund

The Insurance Fund is a fund used as insurance for the protocol. Any bad debt must be covered by the insurance fund so creditors can be reimbursed - bad debt can occur if there is an imbalance between long and short positions during times of large price movements.

### Fee Pool

The Fee Pool receives fees from the transactions in the Engine, and then distributes them to our token holders according to their share of the total token supply. The Fee Pool can support multiple denominations of token.

### Pricefeed

The Pricefeed contract is meant as an integration contract to interface with the oracle and provide accurate external prices to the vAMM. Now, this is for funding payments only, but in the future it might allow for rebasing the vAMM to improve pricing in the vAMM.
