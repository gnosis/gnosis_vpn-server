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
    #!/usr/bin/env bash
    set -o errexit -o nounset -o pipefail

    priv_key=$(if [ "${PRIVATE_KEY:-}" = "" ]; then wg genkey; else echo "${PRIVATE_KEY}"; fi)

    docker run --rm --detach \
        --env PRIVATE_KEY=${priv_key} \
        --publish 8000:8000 \
        --publish 51821:51820/udp \
        --cap-add=NET_ADMIN \
        --add-host=host.docker.internal:host-gateway \
        --sysctl net.ipv4.conf.all.src_valid_mark=1 \
        --name gnosis_vpn-server gnosis_vpn-server

# stop docker container
docker-stop:
    docker stop gnosis_vpn-server

# enter docker container interactively
docker-enter:
    docker exec --interactive --tty gnosis_vpn-server-dev bash

# checkout submodules
submodules:
    git submodule update --init --force

# helper to start local cluster from hoprnet submodule
start-cluster:
    #!/usr/bin/env bash
    cd modules/hoprnet
    nix develop .#cluster --command make localcluster-expose1

# run full system test
system-test: submodules docker-build
    #!/usr/bin/env bash
    set -o errexit -o nounset -o pipefail

    cleanup() {
        echo "[CLEANUP] Shutting down cluster"
        # Send SIGINT to the entire process group (negative PID)
        timeout --kill-after=1m 30s kill -INT -- -$CLUSTER_PID

        echo "[CLEANUP] Shutting down server container"
        just docker-stop || true

        echo "[CLEANUP] Shutting down client container"
        cd modules/gnosis_vpn-client && just docker-stop || true

        echo "[CLEANUP] Done"
    }

    trap cleanup SIGINT SIGTERM EXIT


    ####
    ## PHASE 1: ready local cluster

    # 1a: start cluster
    setsid just start-cluster > cluster.log 2>&1 &
    CLUSTER_PID=$!
    echo "[PHASE1] Starting cluster with PID: $CLUSTER_PID"

    # 1b: wait for nodes
    EXPECTED_PATTERN="All nodes ready"
    TIMEOUT_S=300
    ENDTIME=$(($(date +%s) + TIMEOUT_S))
    echo "[PHASE1] Waiting for log pattern: '${EXPECTED_PATTERN}' with ${TIMEOUT_S}s timeout"

    while true; do
        if grep -q "$EXPECTED_PATTERN" cluster.log; then
            echo "[PHASE1] ${EXPECTED_PATTERN}"
            break
        fi
        if [ $(date +%s) -gt $ENDTIME ]; then
            echo "[PHASE1] Timeout reached"
            tail --lines 20 cluster.log
            exit 1
        fi
        sleep 1
    done

    # 1c: extract values
    PEER_ID_LOCAL6=$(awk '/local6/,/Admin UI/ {if ($1 == "Peer" && $2 == "Id:") print $3}' cluster.log)
    API_TOKEN_LOCAL1=$(awk '/local1/,/Admin UI/ {if ($0 ~ /Admin UI:/) print $0}' cluster.log | sed -n 's/.*apiToken=\(.*\)$/\1/p')
    API_PORT_LOCAL1=$(awk '/local1/,/Rest API/ {if ($1 == "Rest" && $2 == "API:") print $3}' cluster.log | sed -n 's|.*:\([0-9]\+\)/.*|\1|p')

    echo "[PHASE1] Peer ID (local6): $PEER_ID_LOCAL6"
    echo "[PHASE1] API Token (local1): $API_TOKEN_LOCAL1"
    echo "[PHASE1] API Port (local1): $API_PORT_LOCAL1"


    ####
    ## PHASE 2: ready gnosis_vpn-server

    # 2a: start server
    SERVER_PRIVATE_KEY=$(wg genkey)
    echo "[PHASE2] Starting gnosis_vpn-server with public key: #(echo $SERVER_PRIVATE_KEY | wg pubkey)"
    SERVER_PRIVATE_KEY=$SERVER_PRIVATE_KEY just docker-run

    # 2b: wait for server
    EXPECTED_PATTERN="Rocket has launched"
    TIMEOUT_S=300
    ENDTIME=$(($(date +%s) + TIMEOUT_S))
    echo "[PHASE2] Waiting for log pattern: '${EXPECTED_PATTERN}' with ${TIMEOUT_S}s timeout"

    while true; do
        if docker logs gnosis_vpn-server | grep -q "$EXPECTED_PATTERN"; then
            echo "[PHASE2] ${EXPECTED_PATTERN}"
            break
        fi
        if [ $(date +%s) -gt $ENDTIME ]; then
            echo "[PHASE2] Timeout reached"
            docker logs --tail 20 gnosis_vpn-server
            exit 2
        fi
        sleep 1
    done

    # 2c: register client key
    CLIENT_PRIVATE_KEY=$(wg genkey)
    CLIENT_WG_IP=$(curl --silent -H "Accept: application/json" -H "Content-Type: application/json" \
            -d "{\"public_key\": \"$(echo $CLIENT_PRIVATE_KEY | wg pubkey)\"}" \
            localhost:8000/api/v1/clients/register | jq -r .ip)

    echo "[PHASE2] Client Wireguard IP: $CLIENT_WG_IP"


    ####
    ## PHASE 3: ready gnosis_vpn-client

    # 3a: start client
    pushd modules/gnosis_vpn-client
    just docker-build
    ADDRESS="${CLIENT_WG_IP}/32" DESTINATION_PEER_ID="${PEER_ID_LOCAL6}" API_TOKEN="${API_TOKEN_LOCAL1}" \
      API_PORT="${API_PORT_LOCAL1}" PRIVATE_KEY="${CLIENT_PRIVATE_KEY}" \
      SERVER_PUBLIC_KEY="$(echo $SERVER_PRIVATE_KEY | wg pubkey)" just docker-run
    popd

    # 3b: wait for client to connect

    sleep 6000
    exit 0
