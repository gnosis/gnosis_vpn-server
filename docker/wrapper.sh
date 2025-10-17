#!/usr/bin/env bash

set -o errexit

if [ ! -f /app/privatekey ]; then
  echo "WG_PRIVATE_KEY is not set - generating a new one"
  wg genkey >/app/privatekey
fi

./gnosis_vpn-server --config-file ./config.toml serve --periodically-run-cleanup --sync-wg-interface
