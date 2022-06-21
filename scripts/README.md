# Deployment and Scenario Scripts

This directory contains a number of scripts that enable the deployment of Margined Protocol contracts to both local and external networks. Additionally, included are a number of scripts that run scenarios on the deployed smart contracts.

## Pre-Requisites

In order to run the scripts locally you must:

* Have followed all the instructions contained in the README of this repository
* Installed and built [LocalTerra](https://github.com/terra-money/LocalTerra)
* Javascript / Node Environment
  * Node v16.14.2
  * npm 6.14.7

## Deploy and Run Locally

1. Launch local network

2. Build artifacts

**Note:** run using sudo so that additional files can be written to the `artifacts` directory created by docker.

```
sudo ./scripts/build_artifacts.sh
```

1. Deploy the contracts to local cosmwasm testnet:

First enter the scripts directory:
```
cd scripts
```

Then install npm packages and run deployment script:

```
npm install
node --loader ts-node/esm deploy.ts
```
