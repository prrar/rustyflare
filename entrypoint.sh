#!/bin/sh

INTERVAL="${CF_INTERVAL:-300}"

while true; do
    /rustyflare
    echo $? > /tmp/status
    sleep "$INTERVAL"
done
