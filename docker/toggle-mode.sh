#!/bin/sh
# Flip the whole running topology between two modes:
#
#   kernel  - the switch container bridges its three ports (eth0/eth1/eth2)
#             into one real L2 segment. The switch binary is killed (the
#             bridge does forwarding). Kernel ARP stays OFF everywhere -
#             only the user's boot/host code handles ARP.
#
#   custom  - the bridge is torn down (back to three isolated
#             point-to-point links). Kernel ARP is turned off everywhere;
#             nothing works except whatever your own raw-socket ARP/relay
#             code does.
#
# The nodes keep their docker-assigned IPs (10.0.1.10 / 10.0.2.10 /
# 10.0.3.10) in both modes - those are the addresses your ARP code
# operates on.
#
# Usage: ./toggle-mode.sh [kernel|custom]   (default: custom)
#
# Safe to run repeatedly / in either starting state - every step is
# best-effort (|| true) so it doesn't matter what state you're coming from.

set -e

MODE="${1:-custom}"
case "$MODE" in
    kernel|custom) ;;
    *)
        echo "Usage: $0 [kernel|custom]" >&2
        exit 1
        ;;
esac

SWITCH="switch"
SWITCH_IFACES="eth0 eth1 eth2"
BRIDGE="br0"

NODES="alice bob carol"
NODE_IFACE="eth0"

if [ "$MODE" = "kernel" ]; then
    echo "$SWITCH: bridging $SWITCH_IFACES into $BRIDGE"
    docker exec "$SWITCH" ip link add name "$BRIDGE" type bridge 2>/dev/null || true
    docker exec "$SWITCH" ip link set "$BRIDGE" up
    for i in $SWITCH_IFACES; do
        docker exec "$SWITCH" ip link set "$i" master "$BRIDGE"
        docker exec "$SWITCH" ip link set "$i" up
    done

    # Kill the switch binary - the kernel bridge does forwarding now.
    docker exec "$SWITCH" bash -c 'kill $(cat /tmp/switch.pid)' 2>/dev/null || true

    # ARP stays OFF on all nodes - only the user's code handles ARP.
    for c in $NODES; do
        echo "$c: arp off (user ARP only)"
        docker exec "$c" ip link set "$NODE_IFACE" arp off
    done

    # Re-send gratuitous ARP now that the bridge is up so all nodes learn each other.
    for c in $NODES; do
        echo "$c: sending gratuitous ARP"
        docker exec "$c" /app/boot 2>&1 || true
    done

    echo "Done. Bridged; kernel ARP off - user ARP code only."
else
    echo "$SWITCH: unbridging $SWITCH_IFACES, removing $BRIDGE"
    for i in $SWITCH_IFACES; do
        docker exec "$SWITCH" ip link set "$i" nomaster 2>/dev/null || true
        docker exec "$SWITCH" ip link set "$i" arp off
    done
    docker exec "$SWITCH" ip link delete "$BRIDGE" 2>/dev/null || true

    for c in $NODES; do
        echo "$c: arp off"
        docker exec "$c" ip link set "$NODE_IFACE" arp off
        # Ensure /16 on-link so all nodes see each other as directly attached
        cur="$(docker exec "$c" ip -4 -o addr show dev "$NODE_IFACE" | awk '{print $4}' | head -1)"
        if [ -n "$cur" ]; then
            base="${cur%/*}"
            docker exec "$c" ip addr del "$cur" dev "$NODE_IFACE" 2>/dev/null || true
            docker exec "$c" ip addr add "${base}/16" dev "$NODE_IFACE"
            echo "  $c: set ${base}/16 on $NODE_IFACE"
        fi
    done

    echo "Done. Kernel ARP is off - your raw-socket implementation owns ARP now."
fi
