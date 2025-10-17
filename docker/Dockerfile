FROM alpine:3.22

# enable wireguard
RUN \
    apk add --no-cache --update wireguard-tools iptables && \
    rm -rf /tmp/*

WORKDIR /app

# add local files
COPY ./wg0.conf /etc/wireguard/wg0.conf
COPY ./config.toml config.toml
COPY ./gnosis_vpn-server gnosis_vpn-server
COPY ./wrapper.sh wrapper.sh

# port handling
EXPOSE 8000/tcp
EXPOSE 51820/udp

CMD ["./wrapper.sh"]
