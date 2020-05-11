FROM rustlang/rust:nightly
ARG PORT=8000

WORKDIR /var/app
ADD . .
RUN cargo build --release

CMD ./target/release/musicbot-registry
HEALTHCHECK --interval=5m --timeout=3s \
  CMD curl -f http://localhost:$PORT/ || exit 1
