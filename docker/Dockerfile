FROM alpine:3.21

# enable wireguard
# RUN \
    # apk add wireguard-tools-wg-quick iptables wireguard-tools iproute2 && \
    # rm -rf /tmp/*

# add local files
COPY ./config.toml config.toml
COPY ./wggvpn.conf wggvpn.conf
COPY ./gnosis_vpn-server gnosis_vpn-server

WORKDIR /app

# port handling
EXPOSE 51820/udp

CMD ["./gnosis_vpn-server", "-config", "config.toml", "serve", "--periodically-run-cleanup", "--sync-wg-interface"]
