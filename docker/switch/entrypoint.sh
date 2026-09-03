#!/bin/sh
set -e

PORT_A="${SWITCH_PORT_A:-eth0}"
PORT_B="${SWITCH_PORT_B:-eth1}"
PORT_C="${SWITCH_PORT_C:-eth2}"
ARP_OFF="${ARP_OFF:-1}"

# Give each port a fixed, recognizable MAC (set SWITCH_MAC_A/B/C to override).
for pair in "$PORT_A:$SWITCH_MAC_A" "$PORT_B:$SWITCH_MAC_B" "$PORT_C:$SWITCH_MAC_C"; do
    iface="${pair%%:*}"
    mac="${pair#*:}"
    if [ -n "$mac" ]; then
        ip link set "$iface" down
        ip link set "$iface" address "$mac"
        ip link set "$iface" up
    fi
done

# Whatever you build here relays raw frames itself; the kernel's ARP
# module doesn't need to answer on any port. Set ARP_OFF=0 to leave
# the kernel's ARP handling on instead.
if [ "$ARP_OFF" != "0" ]; then
    ip link set "$PORT_A" arp off || true
    ip link set "$PORT_B" arp off || true
    ip link set "$PORT_C" arp off || true
fi

# Run switch in background so it can be killed (e.g. in kernel mode
# where the bridge replaces it) without stopping the container.
/app/switch &
echo $! > /tmp/switch.pid
exec tail -f /dev/null
