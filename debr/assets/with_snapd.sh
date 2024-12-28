#!/bin/bash

SNAPD_PATH="/usr/lib/snapd/snapd"

echo "Setting snapd socket permissions"
chmod 0666 "/run/snapd.socket"
chmod 0666 "/run/snapd-snap.socket"

echo "Starting snapd \"service\" job"
$SNAPD_PATH & 
SNAPD_PID=$!
echo "snapd started with PID $SNAPD_PID"

echo "Delay, waiting for snapd to initialize"
sleep 5

$(WITH_SNAPD)

echo "Delay, waiting for snapd to process"
sleep 5

# Stop the snapd service (kill the background snapd process)
echo "Stopping snapd \"service\" job"
kill $SNAPD_PID
echo "snapd service stopped."
