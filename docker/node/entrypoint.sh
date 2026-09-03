#!/bin/sh
set -e

IFACE="${ARP_IFACE:-eth0}"
ARP_OFF="${ARP_OFF:-1}"

# Give the interface a fixed, recognizable MAC (set ARP_MAC to override).
if [ -n "$ARP_MAC" ]; then
    ip link set "$IFACE" down
    ip link set "$IFACE" address "$ARP_MAC"
    ip link set "$IFACE" up
fi

# Disable the kernel's own ARP handling on this interface so it can't
# answer requests or populate its neighbor cache behind your back -
# your implementation owns ARP on this link exclusively. Set ARP_OFF=0
# to leave the kernel's ARP handling on instead.
if [ "$ARP_OFF" != "0" ]; then
    ip link set "$IFACE" arp off || true
fi

# Widen the node's on-link subnet from docker's /24 to /16 so all three
# nodes (10.0.1.10 / 10.0.2.10 / 10.0.3.10) fall in the same 10.0.0.0/16
# L2-reachable range. This replaces the mask on the SAME single address
# (del then add) - it never introduces a second IP, so the host.rs
# getifaddrs().next() read keeps returning the same address.
CUR_IP="$(ip -4 -o addr show dev "$IFACE" | awk '{print $4}' | head -1)"
if [ -n "$CUR_IP" ]; then
    BASE_IP="${CUR_IP%/*}"
    ip addr del "$CUR_IP" dev "$IFACE" 2>/dev/null || true
    ip addr add "${BASE_IP}/16" dev "$IFACE"
    echo "Set $IFACE to ${BASE_IP}/16 for /16 on-link routing"
fi

# Start host in background so it's listening before boot sends.
/app/host &
HOST_PID=$!

# Run boot to send gratuitous ARP to peers (others' host will learn us)
if [ -x /app/boot ]; then
    echo "Running ARP boot discovery..."
    /app/boot
fi

# Bring host to foreground and wait
wait $HOST_PID
