#!/usr/bin/env bash

set -e
set -o pipefail

docker run --rm -v "$(pwd)":/code \
    --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
    --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
    cosmwasm/workspace-optimizer:0.12.5

wget -O ./artifacts/cw20_base.wasm https://github.com/cosmwasm/cw-plus/releases/download/v0.10.2/cw20_base.wasm
