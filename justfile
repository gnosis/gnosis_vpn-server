default:
    @just --list

# build static x86_64 linux binary via nix
build:
    nix build .#binary-gnosis_vpn-server-x86_64-linux

# build docker image (runs build first)
docker-build: build
    #!/usr/bin/env bash
    set -o errexit -o nounset -o pipefail

    cp result/bin/gnosis_vpn-server docker/
    chmod 775 docker/gnosis_vpn-server
    docker build --platform linux/x86_64 -t gnosis_vpn-server docker/

# run docker container detached (requires PRIVATE_KEY env var)
docker-run:
    #!/usr/bin/env bash
    set -o errexit -o nounset -o pipefail

    log_level=$(if [ "${RUST_LOG:-}" = "" ]; then echo info; else echo "${RUST_LOG}"; fi)

    docker run --rm --detach \
        --env PRIVATE_KEY=${PRIVATE_KEY:-} \
        --env RUST_LOG=${log_level} \
        --publish 8000:8000 \
        --publish 51821:51820/udp \
        --cap-add=NET_ADMIN \
        --add-host=host.docker.internal:host-gateway \
        --sysctl net.ipv4.conf.all.src_valid_mark=1 \
        --name gnosis_vpn-server gnosis_vpn-server

# stop the running docker container
docker-stop:
    docker stop gnosis_vpn-server

# open a shell inside the running docker container
docker-enter:
    docker exec --interactive --tty gnosis_vpn-server bash
