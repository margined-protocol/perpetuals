# Margin Protocol virtual Automated Market Maker (vAMM)

This repo contains a template vAMM for use on Terra blockchain.

## To Do List

- [ ] vAMM
    - [ ] Initialise
        - [x] Owner, assets, etc
    - [ ] Execute
        - [ ] Init vAMM (define constant product func k)
            - define the state of the new vAMM
        - [ ] Long / Short
    - [ ] Query
        - Latest Price
- [ ] Margin Engine
    - [ ] Initialise
        - [x] owner, vAMM, etc
    - [ ] Execute
        - [ ] New position / Close position
        - [ ] New eligible collateral (maybe? potentially we only allow a single type? would make x-margin easier)
    - [ ] Query
- [ ] Factory
- [ ] Governance
    

## Quickstart

TODO

## Reading / Docs

* [Perpetual Protocol](https://docs.perp.fi/getting-started/how-it-works/trading)
* [Audaces Protocol](https://docs.bonfida.org/collection/v/help/audaces-perpetuals/white-paper)
* [Example MultiTest](https://github.com/astroport-fi/astroport-core/blob/c0ab5440300102498b025b8d3aedb7cf22ac5800/contracts/factory/tests/integration.rs)
