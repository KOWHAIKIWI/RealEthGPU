#!/bin/bash

echo "🚀 Killing any old miners..."
killall seeds

sleep 2

echo "🛡️ Launching Watchdog for all miners..."
TOTAL_WORKERS=14 ./launch_watchdog.sh &
