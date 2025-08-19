#!/usr/bin/env bash

set -o errexit

declare key="$PRIVATE_KEY"
if [ -z "$key" ]; then
  echo "PRIVATE_KEY is not set - generating a new one"
  key=$(wg genkey)
fi

awk -v key="$key" '{gsub(/PrivateKey = <private key>/, "PrivateKey = " key); print}' wggvpn.conf >temp.conf && mv temp.conf wggvpn.conf

chmod 600 wggvpn.conf
squid -f ./squid.conf
./gnosis_vpn-server --config-file ./config.toml serve --periodically-run-cleanup --sync-wg-interface
