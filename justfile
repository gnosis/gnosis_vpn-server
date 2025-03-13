alias b := build

# build static linux binary
build:
    nix build .#gvpn-x86_64-linux
