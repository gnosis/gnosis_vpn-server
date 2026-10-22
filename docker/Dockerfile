# alpine 3.24.3 (digest of the alpine:3.24 tag, rebuilt 2026-10-21)
FROM alpine:3.24@sha256:71e9b7bae160ba167fa17414401b94cbcdaf3c32feeab9298bd794a2a0614c35

# enable wireguard
RUN \
    apk add --no-cache --update wireguard-tools iptables && \
    rm -rf /tmp/*

WORKDIR /app

# add local files
COPY ./config.toml config.toml
COPY ./wggvpn.conf wggvpn.conf
COPY ./gnosis_vpn-server gnosis_vpn-server
COPY ./wrapper.sh wrapper.sh

# port handling
EXPOSE 8000/tcp
EXPOSE 51820/udp

CMD ["./wrapper.sh"]
