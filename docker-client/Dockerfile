FROM alpine:3.21

# enable wireguard
RUN \
    apk add --no-cache --update wireguard-tools && \
    rm -rf /tmp/*

WORKDIR /app

# add local files
COPY ./wgclient.conf wgclient.conf
COPY ./monitor.sh monitor.sh

# port handling
EXPOSE 51820/udp

CMD ["./monitor.sh"]
