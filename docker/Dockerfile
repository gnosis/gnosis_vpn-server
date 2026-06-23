FROM alpine:3.24@sha256:28bd5fe8b56d1bd048e5babf5b10710ebe0bae67db86916198a6eec434943f8b

# enable wireguard
RUN \
    apk add --no-cache --update wireguard-tools && \
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
