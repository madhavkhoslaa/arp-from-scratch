#!/bin/sh
# Flip the whole running topology between two modes:
#
#   kernel  - the switch container bridges its three ports (eth0/eth1/eth2)
#             into one real L2 segment, alice/bob/carol get their logical
#             192.169.69.x/24 address actually assigned to eth0, and kernel
#             ARP is turned on everywhere. Result: real `ping`/ARP between
#             alice, bob and carol works, mediated entirely by the kernel.
#             Good for sanity-checking the topology/addressing itself.
#
#   custom  - the bridge is torn down (back to three isolated
#             point-to-point links, matching the original topology),
#             the 192.169.69.x address is removed from each node's
#             interface (ARP_MY_IP/ARP_PEER_IPS remain as env vars only),
#             and kernel ARP is turned off everywhere. Nothing works
#             except whatever your own raw-socket ARP/relay code does.
#
# Usage: ./toggle-mode.sh kernel|custom
#
# Safe to run repeatedly / in either starting state - every step is
# best-effort (|| true) so it doesn't matter what state you're coming from.

set -e

MODE="$1"
case "$MODE" in
    kernel|custom) ;;
    *)
        echo "Usage: $0 kernel|custom" >&2
        exit 1
        ;;
esac

SWITCH="switch"
SWITCH_IFACES="eth0 eth1 eth2"
BRIDGE="br0"

NODES="alice bob carol"
NODE_IFACE="eth0"
# must match ARP_MY_IP in docker-compose.yml
node_ip() {
    case "$1" in
        alice) echo "192.169.69.10/24" ;;
        bob)   echo "192.169.69.20/24" ;;
        carol) echo "192.169.69.30/24" ;;
    esac
}

if [ "$MODE" = "kernel" ]; then
    echo "$SWITCH: bridging $SWITCH_IFACES into $BRIDGE"
    docker exec "$SWITCH" ip link add name "$BRIDGE" type bridge 2>/dev/null || true
    docker exec "$SWITCH" ip link set "$BRIDGE" up
    for i in $SWITCH_IFACES; do
        docker exec "$SWITCH" ip link set "$i" master "$BRIDGE"
        docker exec "$SWITCH" ip link set "$i" up
    done

    for c in $NODES; do
        ip="$(node_ip "$c")"
        echo "$c: assigning $ip to $NODE_IFACE, arp on"
        docker exec "$c" ip addr add "$ip" dev "$NODE_IFACE" 2>/dev/null || true
        docker exec "$c" ip link set "$NODE_IFACE" arp on
    done

    echo "Done. Try: docker exec alice ping -c 3 192.169.69.20"
else
    echo "$SWITCH: unbridging $SWITCH_IFACES, removing $BRIDGE"
    for i in $SWITCH_IFACES; do
        docker exec "$SWITCH" ip link set "$i" nomaster 2>/dev/null || true
        docker exec "$SWITCH" ip link set "$i" arp off
    done
    docker exec "$SWITCH" ip link delete "$BRIDGE" 2>/dev/null || true

    for c in $NODES; do
        ip="$(node_ip "$c")"
        echo "$c: removing $ip from $NODE_IFACE, arp off"
        docker exec "$c" ip addr del "$ip" dev "$NODE_IFACE" 2>/dev/null || true
        docker exec "$c" ip link set "$NODE_IFACE" arp off
    done

    echo "Done. Kernel is out of the way - your raw-socket implementation owns ARP now."
fi
