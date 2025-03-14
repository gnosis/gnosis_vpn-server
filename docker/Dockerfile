FROM alpine:3.21

# enable wireguard
RUN \
    apk add wireguard-tools && \
    rm -rf /tmp/*

WORKDIR /app

# add local files
COPY ./config.toml config.toml
COPY ./wggvpn.conf wggvpn.conf
COPY ./gnosis_vpn-server gnosis_vpn-server

# port handling
EXPOSE 51822/udp

CMD ["./gnosis_vpn-server", "--config-file", "./config.toml", "serve", "--periodically-run-cleanup", "--sync-wg-interface"]
