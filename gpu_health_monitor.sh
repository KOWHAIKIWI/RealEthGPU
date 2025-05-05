#!/bin/bash

# Settings
DANGER_TEMP=85  # degrees Celsius
CHECK_INTERVAL=30  # seconds

echo "🚨 GPU Health Monitor running... Shutdown at ${DANGER_TEMP}C"

while true; do
    OVERHEAT=false

    for TEMP in $(nvidia-smi --query-gpu=temperature.gpu --format=csv,noheader,nounits); do
        if [ "$TEMP" -ge "$DANGER_TEMP" ]; then
            echo "⚠️ GPU Overheat detected at ${TEMP}C! Shutting down miners..."
            killall seeds
            OVERHEAT=true
            break
        fi
    done

    if [ "$OVERHEAT" = true ]; then
        touch overheated.flag
        exit 0
    fi

    sleep $CHECK_INTERVAL
done
