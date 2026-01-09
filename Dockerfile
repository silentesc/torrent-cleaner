###########
# Builder #
###########
FROM rust:1.92-alpine AS builder

RUN apk add --no-cache build-base musl-dev pkgconfig openssl-dev sqlite-dev ca-certificates

WORKDIR /app

COPY . .

ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN cargo build --release --locked

##########
# Runner #
##########
FROM alpine:3.23.2

RUN apk add --no-cache ca-certificates openssl sqlite-libs libgcc

# Default envs (can be overridden in docker-compose)
ENV PUID=1000 \
    PGID=1000 \
    TZ=Etc/UTC

# Create a group & user that match host IDs
RUN addgroup -g ${PGID} appgroup \
    && adduser  -D -u ${PUID} -G appgroup appuser

WORKDIR /app

COPY --from=builder /app/target/release/torrent-cleaner /app/torrent-cleaner

# Create config
RUN mkdir -p /config \
    && chown -R appuser:appgroup /app /config /app/torrent-cleaner

# Switch to non-root user
USER appuser

ENTRYPOINT ["/app/torrent-cleaner"]
