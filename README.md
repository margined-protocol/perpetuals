# Margined Protocol Perpertuals

[![Continuous Integration](https://github.com/margined-protocol/mrgnd-perpetuals/actions/workflows/ci.yml/badge.svg)](https://github.com/margined-protocol/mrgnd-perpetuals/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/margined-protocol/mrgnd-perpetuals/branch/main/graph/badge.svg?token=FDMKT04UWK)](https://codecov.io/gh/margined-protocol/mrgnd-perpetuals)


This repo contains a the Margined Protocol a decentralized perpetual contract protocol on the Terra Blockchain.

## Contracts

| Contract                                                | Reference | Description                                                                                           |
| ------------------------------------------------------- | --------- | ----------------------------------------------------------------------------------------------------- |
| [`Margin Engine`](./contracts/margined-engine)          | [doc]()   | Margin engine that manages users positions and the collateral management                              |
| [`vAMM`](./contracts/margined-vamm)                     | [doc]()   | Virtual AMM enabling users to take perpetual positions                                                |
| [`Price Feed`](./contracts/margined-price-feed)         | [doc]()   | Integration contract for the data oracles and other data related logic                                |
| [`Governance`](./contracts/margined-price-feed)         | [doc]()   | TODO                                                                                                  |
| [`Factory`](./contracts/margined-price-feed)            | [doc]()   | TODO                                                                                                  |

## Get started

### Environment Setup

- Rust v1.44.1+
- `wasm32-unknown-unknown` target
- Docker

1. Install `rustup` via https://rustup.rs/

2. Run the following:

```sh
rustup default stable
rustup target add wasm32-unknown-unknown
```

3. Make sure [Docker](https://www.docker.com/) is installed

### Unit / Integration Tests

Each contract contains Rust unit and integration tests embedded within the contract source directories. You can run:

```sh
cargo unit-test
cargo integration-test
```
### Build

Clone this repository and build the source code:
```
git clone git@github.com:margined-protocol/mrgnd-perpetuals.git
cd mrgnd-perpetuals
cargo build
```

## To Do List

- [ ] vAMM
    - [ ] Initialise
        - [x] Owner, assets, etc
    - [ ] Execute
        - [x] Init vAMM (define constant product func k)
            - define the state of the new vAMM
        - [x] Long / Short
        - [ ] SettleFunding
    - [ ] Query
        - [x] Latest Price
- [ ] Margin Engine
    - [x] Initialise
        - [x] owner, vAMM, etc
    - [ ] Execute
        - [x] New position / Close position
        - [ ] New eligible collateral (maybe? potentially we only allow a single type? would make x-margin easier)
        - [ ] Update vAmms, i.e. append, remove etc
        - [ ] Update vAmms, i.e. append, remove etc
    - [ ] Query
- [ ] [Oracle](https://github.com/terra-money/tefi-oracle-contracts)
  - [ ] PriceFeed contract that integrates against TeFi hub
  - [x] Wrapper for TeFi oracles which do calcs listed below
  - [x] TWAP
  - [ ] ???
- [ ] Decimal Library
  - General decimal calculation library for use around with my fixed point decimals
- [ ] Factory
- [ ] Governance
- [ ] General
  - [ ] Testing framework improvements
    - Wrapper for smart contract functions
    - Setup files
    - Better organisation
  - [ ] Code comment documentation
  - [x] Code Coverage - cargo-tarpaulin   
  - [x] Code linting

## Reading / Docs

* [Perpetual Protocol](https://docs.perp.fi/getting-started/how-it-works/trading)
* [Audaces Protocol](https://docs.bonfida.org/collection/v/help/audaces-perpetuals/white-paper)
* [Perpetuals In-Depth](https://0xkowloon.substack.com/p/dissecting-the-perpetual-protocol)
* [Dawn of Decentralised Derivative](https://members.delphidigital.io/reports/the-dawn-of-decentralized-derivatives/)
