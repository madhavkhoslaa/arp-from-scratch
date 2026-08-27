#!/bin/sh
# Flush the kernel's ARP/neighbor cache on every container in the topology.
#
# Only meaningful in `kernel` mode (see toggle-mode.sh) - that's the only
# time the kernel is populating a neighbor cache at all. In `custom` mode
# the interfaces are NOARP, so there's nothing here for the kernel to
# cache; any resolution your own raw-socket implementation does lives in
# your own program's memory, not `ip neigh`.
#
# Usage: ./flush-arp.sh

set -e

CONTAINERS="alice bob carol switch"

for c in $CONTAINERS; do
    echo "$c: flushing neighbor cache"
    docker exec "$c" ip neigh flush all
done

echo "Done."
