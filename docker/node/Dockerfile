FROM alpine:3.20

RUN apk add --no-cache gcc musl-dev make iproute2 tcpdump

WORKDIR /app
COPY entrypoint.sh ./
RUN chmod +x entrypoint.sh

# Your source is bind-mounted at /app/src (see docker-compose.yml).
# Build/run it yourself, e.g.: docker exec -it alice sh
CMD ["sleep", "infinity"]
ENTRYPOINT ["./entrypoint.sh"]
