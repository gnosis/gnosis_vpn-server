# build static linux binary
build:
    nix build .#gvpn-x86_64-linux

# build docker image
docker: build
    cp result/bin/gnosis_vpn-server docker/
    chmod 775 docker/gnosis_vpn-server
    docker build --platform linux/x86_64 -t gnosis_vpn-server docker

# run docker container
docker-run: docker
    docker run --rm -p 8000:8000 --cap-add=NET_ADMIN --name gnosis_vpn-server-dev gnosis_vpn-server

# enter docker container interactively
docker-enter: docker
    docker run --rm -p 8000:8000 --cap-add=NET_ADMIN --name gnosis_vpn-server-dev -it --entrypoint bash gnosis_vpn-server

# setup server
setup: docker
