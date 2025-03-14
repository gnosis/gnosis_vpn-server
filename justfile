alias b := build

# build static linux binary
build:
    nix build .#gvpn-x86_64-linux

# build docker container
docker: build
    cp result/bin/gnosis_vpn-server docker/
    chmod 775 docker/gnosis_vpn-server
    docker build --platform linux/x86_64 -t gnosis_vpn-server docker

# setup server
setup: docker
