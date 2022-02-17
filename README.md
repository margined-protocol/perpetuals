# Margined Protocol Perpertuals

[![Continuous Integration](https://github.com/shapeshed/mrgnd-perpetuals/actions/workflows/ci.yml/badge.svg)](https://github.com/shapeshed/mrgnd-perpetuals/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/shapeshed/mrgnd-perpetuals/branch/main/graph/badge.svg?token=OXwMwRifUv)](https://codecov.io/gh/shapeshed/mrgnd-perpetuals)

This repo contains a perpetual protocol for use on CosmWasm blockchains.

## Quickstart

TODO

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
        - Latest Price
- [ ] Margin Engine
    - [x] Initialise
        - [x] owner, vAMM, etc
    - [ ] Execute
        - [ ] New position / Close position
        - [ ] New eligible collateral (maybe? potentially we only allow a single type? would make x-margin easier)
        - [ ] Update vAmms, i.e. append, remove etc
        - [ ] Update vAmms, i.e. append, remove etc
    - [ ] Query
- [ ] [Oracle](https://github.com/terra-money/tefi-oracle-contracts)
  - [ ] PriceFeed contract that integrates against TeFi hub
  - [ ] Wrapper for TeFi oracles which do calcs listed below
  - [ ] TWAP
  - [ ] ???
- [ ] Factory
- [ ] Governance
- [ ] General
  - [ ] Code comment documentation
  - [ ] Code Coverage - cargo-tarpaulin   

## Reading / Docs

* [Perpetual Protocol](https://docs.perp.fi/getting-started/how-it-works/trading)
* [Audaces Protocol](https://docs.bonfida.org/collection/v/help/audaces-perpetuals/white-paper)
* [Perpetuals In-Depth](https://0xkowloon.substack.com/p/dissecting-the-perpetual-protocol)
