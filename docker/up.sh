#!/bin/sh
# Build the Rust binaries (if needed) and bring the ARP topology up.
# The containers bind-mount the built binaries, so they must exist and be
# runnable before starting. Container base images are glibc (Debian), matching
# the host build.
#
# Usage: ./up.sh
#   builds arp/{host,switch} then `docker compose up -d --build`

set -e

cd "$(dirname "$0")"

CRATE="${CRATE:-../arp}"
DEBUG_DIR="$CRATE/target/debug/host"   # single binary built by `cargo build`
SWITCH_BIN="$CRATE/target/debug/switch"

echo "==> Building the switch binary"
(cd "$CRATE" && cargo build --bin switch)

echo "==> Building the host binary"
(cd "$CRATE" && cargo build --bin host)

echo "==> Verifying binaries are dynamically loaded against glibc (base image is Debian)"
for b in "$SWITCH_BIN" "$DEBUG_DIR"; do
    if ! ldd "$b" 2>/dev/null | grep -q "libc.so.6"; then
        echo "WARNING: $b does not appear to link against glibc;" >&2
        echo "         the containers use Debian (glibc) and may fail to run it." >&2
    fi
done

echo "==> Bringing the topology up (--build, detached)"
docker compose up -d --build

echo "==> Running containers:"
docker compose ps

echo
echo "Done. Tail a node's raw ARP output with, e.g.:"
echo "  docker compose logs -f alice"
