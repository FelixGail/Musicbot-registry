FROM ekidd/rust-musl-builder:nightly-2020-10-08 as builder

RUN USER=root cargo new --bin musicbot-registry
WORKDIR ./musicbot-registry

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN sudo chown -R $(whoami) /home/rust
RUN cargo update
RUN cargo build --release
RUN rm src/*.rs

COPY ./src/ ./src/
RUN cargo build --release

FROM alpine:latest
ARG APP=/app

ENV TZ=Etc/UTC

RUN apk update \
    && apk add --no-cache ca-certificates tzdata\
    && rm -rf /var/cache/apk/*

COPY --from=builder /home/rust/src/musicbot-registry/target/x86_64-unknown-linux-musl/release/musicbot-registry ${APP}/musicbot-registry
COPY ./Rocket.toml ./Rocket.toml

WORKDIR ${APP}
CMD ["./musicbot-registry"]