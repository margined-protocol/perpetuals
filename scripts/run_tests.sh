#!/bin/bash

set -e

main() {
  if [[ -z $LOCAL_TERRA_REPO_PATH ]]; then
    echo "LOCAL_TERRA_REPO_PATH must be set"
    return 1
  fi

  tests=$(ls tests/*.ts)
  echo $tests

  # ensure LocalTerra is stopped
  docker compose -f $LOCAL_TERRA_REPO_PATH/docker-compose.yml down

  # start LocalTerra
  sed -E -i .bak '/timeout_(propose|prevote|precommit|commit)/s/[0-9]+m?s/200ms/' $LOCAL_TERRA_REPO_PATH/config/config.toml
  docker compose -f $LOCAL_TERRA_REPO_PATH/docker-compose.yml up -d

  # run tests
  for test in $tests; do
    echo Running $test
    node --loader ts-node/esm $test
  done

  docker compose -f $LOCAL_TERRA_REPO_PATH/docker-compose.yml down
}

main