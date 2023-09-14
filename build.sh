#!/usr/bin/env bash
set -o errexit
set -o pipefail
set -o nounset
#set -o xtrace

cd "$(dirname "${BASH_SOURCE[0]}")"

contracts=(
  nft
  organization
)

for contract in "${contracts[@]}"; do
  echo -e "Building ${contract}\n"
  # build
  cargo contract build --target wasm --manifest-path contracts/${contract}/Cargo.toml --release
  # copy contract
  cp --force target/ink/${contract}/${contract}.contract target
  echo -e "\n"
done

echo Done.
