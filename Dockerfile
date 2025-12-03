###########
# Builder #
###########
FROM rust:1.91 AS builder

WORKDIR /app

COPY . .

RUN cargo build --release

##########
# Runner #
##########
FROM debian:stable-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Default envs (can be overridden in docker-compose)
ENV PUID=1000 \
    PGID=1000 \
    TZ=Etc/UTC

# Create a group & user that match host IDs
RUN groupadd -g ${PGID} appgroup \
    && useradd -u ${PUID} -g appgroup -m appuser

WORKDIR /app

COPY --from=builder /app/target/release/torrent-cleaner ./

# Create config
RUN mkdir -p /config \
    && chown -R appuser:appgroup /app /config

# Switch to non-root user
USER appuser

ENTRYPOINT ["./torrent-cleaner"]
