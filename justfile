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
    docker run --rm --detach --env WG_PRIVATE_KEY=$(wg genkey) \
        --cap-add=NET_ADMIN --publish 8000:8000 --publish 51822:51820/udp \
        --name gnosis_vpn-server-dev gnosis_vpn-server

# enter docker container interactively
docker-enter:
    docker exec --interactive --tty gnosis_vpn-server-dev bash
