FROM ubuntu:latest

WORKDIR /app
COPY ./config.toml config.toml
COPY ./gnosis_vpn-server gnosis_vpn-server
RUN umask 077 && wg genkey > privkey
RUN ip link add dev wggvpn type wireguard
RUN wg set wggvpn private-key privkey listen-port 51820
RUN wg showconf wggvpn > wggvpn.conf

CMD ["./gnosis_vpn-server", "-config", "config.toml", "serve", "--periodically-run-cleanup", "--sync-wg-interface"]
