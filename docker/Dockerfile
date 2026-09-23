FROM alpine:3.24@sha256:294b683cb724975bec92580e1e685676bd4b50bda910ddb8c51d4cabeaec77e6

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
