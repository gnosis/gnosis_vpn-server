#!/usr/bin/env bash

set -o errexit
set -x

declare key="$WG_PRIVATE_KEY"
if [ -z "$key" ]; then
  echo "WG_PRIVATE_KEY is not set"
  exit 1
fi

awk -v key="$key" '{gsub(/PrivateKey = <private key>/, "PrivateKey = " key); print}' wggvpn.conf > temp.conf && mv temp.conf wggvpn.conf

./gnosis_vpn-server --config-file ./config.toml serve --periodically-run-cleanup --sync-wg-interface &
wait
