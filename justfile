# build static linux binary
build:
    nix build .#gvpn-x86_64-linux

# build docker image
docker-build: build
    cp result/bin/gnosis_vpn-server docker/
    chmod 775 docker/gnosis_vpn-server
    docker build --platform linux/x86_64 -t gnosis_vpn-server docker/

# run docker container detached
docker-run:
    docker run --rm --detach \
        --env PRIVATE_KEY=$(wg genkey) \
        --publish 8000:8000 \
        --publish 51821:51820/udp \
        --cap-add=NET_ADMIN \
        --name gnosis_vpn-server gnosis_vpn-server

# enter docker container interactively
docker-enter:
    docker exec --interactive --tty gnosis_vpn-server-dev bash
