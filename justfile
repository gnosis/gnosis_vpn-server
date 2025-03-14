alias b := build

# build static linux binary
build:
    nix build .#gvpn-x86_64-linux


# setup server
setup: build
    cp result/bin/gnosis_vpn-server docker/
    chmod 775 docker/gnosis_vpn-server
