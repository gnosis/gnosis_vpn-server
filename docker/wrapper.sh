#!/usr/bin/env bash

set -o errexit

declare key="$WG_PRIVATE_KEY"
if [ -z "$key" ]; then
  echo "WG_PRIVATE_KEY is not set - generating a new one"
  key=$(wg genkey)
fi

sed -i "s/PrivateKey = <private key>/PrivateKey = ${WG_PRIVATE_KEY}/" wggvpn.conf

chmod 600 wggvpn.conf
./gnosis_vpn-server --config-file ./config.toml serve --periodically-run-cleanup --sync-wg-interface
