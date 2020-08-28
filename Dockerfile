#FROM ekidd/rust-musl-builder:nightly-2020-08-26 AS builder
FROM rust AS builder
WORKDIR /home/rust/mercury/
COPY ./Cargo.toml .
COPY ./Cargo.lock .
RUN mkdir src
RUN echo "fn main() {}" > src/main.rs
RUN cargo build --release

RUN rm src/main.rs
ADD . ./
RUN rm ./target/release/deps/mercury*
RUN cargo build --release


#FROM ekidd/rust-musl-builder:nightly-2020-08-26
FROM rust
EXPOSE 5001
WORKDIR /home/rust/mercury
RUN apt-get update && apt-get install -y cron
RUN echo "*/15 * * * * cd /home/rust/mercury; ./target/release/weatherbit" | crontab
RUN /etc/init.d/cron start
COPY --from=builder /home/rust/mercury/target ./target
COPY --from=builder /home/rust/mercury/templates ./templates
COPY --from=builder /home/rust/mercury/static ./static
COPY --from=builder /home/rust/mercury/settings.yaml .
ENTRYPOINT ["./target/release/mercury"]
