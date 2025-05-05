#!/bin/bash

# Auto-restart watchdog script that monitors miner logs for performance drop
# If any worker's seeds/sec drops below 50 million, the miner is restarted

TOTAL_WORKERS=${TOTAL_WORKERS:-4}
THRESHOLD=50000000  # 50M seeds/sec

function get_speed() {
    tail -n 20 "miner$1.log" 2>/dev/null | grep 'Speed:' | tail -n 1 | \
    awk -F'Speed: ' '{print $2}' | awk '{print $1}' | tr -d ','
}

function restart_miner() {
    TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
    GPU=$1
    WORKER=$2
    echo "⚠️ [$TIMESTAMP] Restarting Miner $WORKER (below threshold)..." | tee -a restart.log
    pkill -f "./target/release/seeds $WORKER"
    sleep 2
    CUDA_VISIBLE_DEVICES=$GPU nohup ./target/release/seeds $WORKER > miner$WORKER.log 2>&1 &
}

while true; do
    echo "⌛ Waiting 60 seconds for miners to warm up..."
    sleep 60
    for ((i=0; i<TOTAL_WORKERS; i++)); do
        speed=$(get_speed $i)
        if [[ -n "$speed" && $speed -lt $THRESHOLD ]]; then
            restart_miner $i $i
        else
            echo "✅ Worker $i OK: $speed seeds/sec"
        fi
    done
    sleep 600  # check every 10 minutes
 done

