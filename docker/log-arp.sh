#!/bin/sh
# Log the ARP tables for alice, bob, and carol.
#
# Usage: ./log-arp.sh [interval]
#   interval - how many seconds between each snapshot (default 5, 0 = single shot)
#
# Prints a timestamped `ip neigh` dump for each node, separated by headers.

set -e

INTERVAL="${1:-5}"

CONTAINERS="alice bob carol"

print_tables() {
    for c in $CONTAINERS; do
        echo "================ $c ================"
        docker exec "$c" ip neigh 2>&1 || echo "(failed to read neighbor table)"
        echo
    done
}

if [ "$INTERVAL" = "0" ]; then
    print_tables
    exit 0
fi

echo "Logging ARP tables every ${INTERVAL}s (Ctrl-C to stop)"
while true; do
    echo "===== $(date -Is) ====="
    print_tables
    sleep "$INTERVAL"
done
