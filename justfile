# build static linux binary
build:
    nix build .#gvpn-x86_64-linux

# build docker image
docker-build: build
    cp result/bin/gnosis_vpn-server docker/
    chmod 775 docker/gnosis_vpn-server
    docker build --platform linux/x86_64 -t gnosis_vpn-server docker/

# run docker container detached
docker-run private_key='':
    #!/usr/bin/env bash
    set -o errexit -o nounset -o pipefail
    PRIVATE_KEY=$(if [ "{{ private_key }}" = "" ]; then wg genkey; else echo "{{ private_key }}"; fi)
    docker run --rm --detach \
        --env PRIVATE_KEY=$PRIVATE_KEY \
        --publish 8000:8000 \
        --publish 51821:51820/udp \
        --cap-add=NET_ADMIN \
        --add-host=host.docker.internal:host-gateway \
        --name gnosis_vpn-server gnosis_vpn-server

# enter docker container interactively
docker-enter:
    docker exec --interactive --tty gnosis_vpn-server-dev bash

# checkout submodules
submodules:
    git submodule update --init --force

start-cluster:
    #!/usr/bin/env bash
    cd modules/hoprnet
    nix develop .#cluster --command make localcluster-expose1

# run full system test
system-test: submodules docker-build
    #!/usr/bin/env bash
    set -o errexit -o nounset -o pipefail

    echo "Starting cluster..."
    just start-cluster > cluster.log 2>&1 &
    CLUSTER_PID=$!
    echo "Started cluster with PID: $CLUSTER_PID"

    tail -f cluster.log &
    TAIL_PID=$!

    sleep 60
    echo "Killing tail..."
    kill $TAIL_PID
    sleep 5
    echo "Killing cluster..."
    kill -INT $CLUSTER_PID

    # CLIENT_PRIVATE_KEY=$(wg genkey)
    # SERVER_PRIVATE_KEY=$(wg genkey)
    # just docker-run private_key=$SERVER_PRIVATE_KEY
    # pushd modules/hoprnet
    # PID=$(nix develop .#cluster --command make localcluster-expose1 &)
    # # IP=$(curl -H "Accept: application/json" -H "Content-Type: application/json" -v -d "{\"public_key\": \"$(echo $privkey | wg pubkey)\"}" localhost:8000/api/v1/clients/register | jq -r .ip)
    # echo $PID
