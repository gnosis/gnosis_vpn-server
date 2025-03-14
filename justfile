alias b := build

# build static linux binary
build:
    nix build .#gvpn-x86_64-linux

# build docker container
docker: build
    cp result/bin/gnosis_vpn-server docker/
    chmod 775 docker/gnosis_vpn-server
    docker build --platform linux/x86_64 -t gnosis_vpn-server docker

docker-run: docker
    docker run --rm -p 8000:8000 --cap-add=NET_ADMIN --name gnosis_vpn-server-dev gnosis_vpn-server

docker-enter: docker
    docker run --rm -p 8000:8000 --cap-add=NET_ADMIN --name gnosis_vpn-server-dev -it --entrypoint sh gnosis_vpn-server

# setup server
setup: docker
