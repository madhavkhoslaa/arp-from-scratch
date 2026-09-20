# arp-from-scratch

Implementing ARP by hand: raw sockets, Ethernet frames, and a userspace
switch, with the kernel's own ARP handling turned off so my code is the
only thing resolving IPs to MAC addresses.

## Topology

```
alice --net-alice-switch--\
                            switch --net-switch-carol-- carol
  bob --net-switch-bob----/
```

Three hosts (alice, bob, carol) and a switch, each in its own Docker
container. The hosts are never on the same Docker network — every
network below has exactly two members, so it behaves as a
point-to-point link. The only path between any two hosts is through the
switch, which has to pick a port per frame instead of just relaying
whatever comes in, which is why it needs a MAC address table (CAM).

All containers are Debian-based with kernel ARP disabled on their
interfaces (`ip link set arp off`) — nothing resolves an IP to a MAC
unless my own code does it.

## What's implemented

- **host** — opens an `AF_PACKET`/`SOCK_RAW` socket, reads raw Ethernet
  frames, filters for ARP (`0x0806`), and:
  - passively learns `sender_ip -> src_mac` from every ARP frame it
    sees and pushes it into the kernel's ARP table
  - replies to a broadcast ARP request if it's asking for this host's IP
- **boot** — sends a single gratuitous ARP on startup so the other
  hosts learn this one immediately, rather than waiting for it to send
  a request
- **switch** — opens a raw socket per port (`eth0`/`eth1`/`eth2`), puts
  each interface into promiscuous mode, and:
  - learns `src_mac -> inbound port` into a MAC table
  - floods broadcast frames out every port except the one it came in on
  - forwards unicast frames out the learned port, or floods if the
    destination MAC isn't known yet

Frame parsing and building is manual — no pcap or packet crate — see
`arp/src/utils/build_arp_reply.rs` for the raw Ethernet + ARP header
layout.

## What's ignored

- CAM memory/aging — the switch's MAC table only grows
- VLANs

## Running it

```sh
cd docker
./up.sh              # builds the host/switch binaries and starts the topology
./toggle-mode.sh custom   # (default) isolated links, only my ARP code works
./toggle-mode.sh kernel   # bridges the switch ports so the kernel does L2 forwarding, for comparison
./log-arp.sh          # tail each node's ARP table
./flush-arp.sh         # clear kernel ARP caches (kernel mode only)
```

`up.sh` builds the Rust binaries on the host and bind-mounts them into
the containers, so a rebuild is just `./up.sh` again.
