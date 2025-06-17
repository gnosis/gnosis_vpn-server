# build static linux binary
build:
    nix build .#gvpn-x86_64-linux

# build docker image
docker-build: build
    #!/usr/bin/env bash
    set -o errexit -o nounset -o pipefail

    cp result/bin/gnosis_vpn-server docker/
    chmod 775 docker/gnosis_vpn-server
    docker build --platform linux/x86_64 -t gnosis_vpn-server docker/

# run docker container detached
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

# stop docker container
docker-stop:
    docker stop gnosis_vpn-server

# enter docker container interactively
docker-enter:
    docker exec --interactive --tty gnosis_vpn-server bash

# checkout submodules
submodules:
    git submodule update --init --force

# helper to start local cluster from hoprnet submodule
start-cluster:
    #!/usr/bin/env bash
    set -o errexit -o nounset -o pipefail
    # if on github hosted runner try to free some extra space (see https://github.com/orgs/community/discussions/25678)
    rm -rf /opt/hostedtoolcache

    cd modules/hoprnet
    nix develop .#cluster --command make localcluster-exposed

# full system setup with system tests: 'mode' can be either 'keep-running' or 'ci-system-test'
system-setup mode='keep-running': submodules docker-build
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
    TIMEOUT_S=$((60 * 50)) # 50 minutes
    ENDTIME=$(($(date +%s) + TIMEOUT_S))
    echo "[PHASE1] Waiting for log '${EXPECTED_PATTERN}' with ${TIMEOUT_S}s timeout"

    # print progress report each minute
    ONGOING_INTERVAL_S=60
    START_TIME=$(date +%s)
    NEXT_REPORT_TIME=$((START_TIME + ONGOING_INTERVAL_S))

    while true; do
        if grep -q "$EXPECTED_PATTERN" cluster.log; then
            echo "[PHASE1] ${EXPECTED_PATTERN}"
            break
        fi
        if [ $(date +%s) -gt $ENDTIME ]; then
            echo "[PHASE1] Timeout reached"
            tail --lines 50 cluster.log
            exit 1
        fi
        if [ $(date +%s) -gt $NEXT_REPORT_TIME ]; then
            NEXT_REPORT_TIME=$((NEXT_REPORT_TIME + ONGOING_INTERVAL_S))
            ELAPSED_TIME=$(($(date +%s) - $START_TIME))
            echo "[PHASE1] Peek cluster log after $((ELAPSED_TIME / 60)) minutes"
            tail --lines 5 cluster.log
        fi
        sleep 1
    done

    # 1c: extract values
    PEER_ID_LOCAL5=$(awk '/local5/,/Admin UI/ {if ($1 == "Peer" && $2 == "Id:") print $3}' cluster.log)
    PEER_ID_LOCAL6=$(awk '/local6/,/Admin UI/ {if ($1 == "Peer" && $2 == "Id:") print $3}' cluster.log)
    API_TOKEN_LOCAL1=$(awk '/local1/,/Admin UI/ {if ($0 ~ /Admin UI:/) print $0}' cluster.log | sed -n 's/.*apiToken=\(.*\)$/\1/p')
    API_PORT_LOCAL1=$(awk '/local1/,/Rest API/ {if ($1 == "Rest" && $2 == "API:") print $3}' cluster.log | sed -n 's|.*:\([0-9]\+\)/.*|\1|p')

    echo "[PHASE1] Peer ID 1 (local5): $PEER_ID_LOCAL5"
    echo "[PHASE1] Peer ID 2 (local6): $PEER_ID_LOCAL6"
    echo "[PHASE1] API Token (local1): $API_TOKEN_LOCAL1"
    echo "[PHASE1] API Port (local1): $API_PORT_LOCAL1"


    ####
    ## PHASE 2: ready gnosis_vpn-server

    # 2a: start server
    echo "[PHASE2] Starting gnosis_vpn-server"
    # container was build as part of the deps
    just docker-run

    # 2b: wait for server
    EXPECTED_PATTERN="Rocket has launched"
    TIMEOUT_S=$((60 * 5)) # 5 minutes
    ENDTIME=$(($(date +%s) + TIMEOUT_S))
    echo "[PHASE2] Waiting for log '${EXPECTED_PATTERN}' with ${TIMEOUT_S}s timeout"

    while true; do
        if docker logs --since 3s gnosis_vpn-server | grep -q "$EXPECTED_PATTERN"; then
            echo "[PHASE2] ${EXPECTED_PATTERN}"
            break
        fi
        if [ $(date +%s) -gt $ENDTIME ]; then
            echo "[PHASE2] Timeout reached"
            docker logs --tail 20 gnosis_vpn-server
            exit 2
        fi
        sleep 2.5
    done

    echo "[PHASE2] Server is ready for testing"

    ####
    ## PHASE 3: ready gnosis_vpn-client

    # 3a: start client
    echo "[PHASE3] Starting gnosis_vpn-client"

    pushd modules/gnosis_vpn-client
        just docker-build
        DESTINATION_PEER_ID_1="${PEER_ID_LOCAL5}" \
        DESTINATION_PEER_ID_2="${PEER_ID_LOCAL6}" \
        API_TOKEN="${API_TOKEN_LOCAL1}" \
        API_PORT="${API_PORT_LOCAL1}" \
        just docker-run
    popd

    exp_client_log() {
        EXPECTED_PATTERN="$1"
        TIMEOUT_S="${2}"
        ENDTIME=$(($(date +%s) + TIMEOUT_S))
        echo "[PHASE3] Waiting for log '${EXPECTED_PATTERN}' with ${TIMEOUT_S}s timeout"

        while true; do
            if docker logs --since 3s gnosis_vpn-client | grep -q "$EXPECTED_PATTERN"; then
                echo "[PHASE3] ${EXPECTED_PATTERN}"
                break
            fi
            if [ $(date +%s) -gt $ENDTIME ]; then
                echo "[PHASE3] Timeout reached"
                docker logs --tail 20 gnosis_vpn-client
                exit 2
            fi
            sleep 2.5
        done
    }

    # 3b: wait for client to be ready
    exp_client_log "enter listening mode" 6
    echo "[PHASE3] Client is ready for testing"

    # 3c: run system tests
    echo "[PHASE3] Checking connect via first local node"
    docker exec gnosis_vpn-client ./gnosis_vpn-ctl connect ${PEER_ID_LOCAL5}
    exp_client_log "VPN CONNECTION ESTABLISHED" 11
    echo "[PHASE3] Checking number of connected slots correct"
    [ 1 = $(docker exec gnosis_vpn-server ./gnosis_vpn-server -c config.toml status --json | jq .slots.connected) ]
    echo "[PHASE3] Checking expiration of inactive clients"
    docker kill gnosis_vpn-client

    # client ping is sent every 5-10 secs with 4 sec timeout
    # server handshake timeout is 15 sec, check interval 16 sec
    sleep 17
    server_status=$(docker exec gnosis_vpn-server ./gnosis_vpn-server -c config.toml status --json)
    [ 1 = $(echo "$server_status" | jq .slots.expired) ] || {
        echo "[PHASE3] ERROR: Expected 1 expired slot, got: $server_status"
        exit 1
    }
    echo "[PHASE3] Checking removal of expired clients"
    sleep 17
    server_status=$(docker exec gnosis_vpn-server ./gnosis_vpn-server -c config.toml status --json)
    [ 10 = $(echo "$server_status" | jq .slots.available) ] || {
        echo "[PHASE3] ERROR: Expected 10 available slots, got: $server_status"
        exit 1
    }

    if [ "{{ mode }}" = "ci-system-test" ]; then
        echo "[SUCCESS] System test completed successfully"
        exit 0
    else
        echo "[PHASE3] System setup complete, keeping components running"
        echo "[PHASE3] Press Ctrl+C to stop the cluster and containers"
        wait $CLUSTER_PID
        exit 0
    fi
