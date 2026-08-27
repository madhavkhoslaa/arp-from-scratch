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

exec "$@"
