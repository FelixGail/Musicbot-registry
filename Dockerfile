FROM rustlang/rust:nightly

WORKDIR /var/app
ADD . .
RUN cargo build --release

CMD ./target/release/musicbot-registry
