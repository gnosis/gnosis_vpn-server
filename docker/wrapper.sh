#!/usr/bin/env bash

set -o errexit

if [ ! -f /etc/wireguard/privatekey ]; then
  echo "WG_PRIVATE_KEY is not set - generating a new one"
  wg genkey >/etc/wireguard/privatekey
fi

./gnosis_vpn-server --config-file ./config.toml serve --periodically-run-cleanup --sync-wg-interface
