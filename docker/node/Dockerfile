FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    iproute2 iputils-ping iputils-arping tcpdump libc6 libgcc-s1 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY entrypoint.sh ./
RUN chmod +x entrypoint.sh

# Binary is bind-mounted at /app/host (see docker-compose.yml)
CMD ["sleep", "infinity"]
ENTRYPOINT ["./entrypoint.sh"]
