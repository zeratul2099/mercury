FROM ekidd/rust-musl-builder:nightly-2020-08-26 AS builder
#FROM rust AS builder
WORKDIR /home/rust/mercury/
RUN sudo chown -R rust:rust .
COPY ./Cargo.toml .
COPY ./Cargo.lock .
RUN mkdir ./src
RUN echo "fn main() {}" > src/main.rs
RUN cargo build --release

RUN rm src/main.rs
ADD . ./
RUN rm ./target/x86_64-unknown-linux-musl/release/deps/mercury*
RUN cargo build --release


FROM debian:latest

ARG APP=/home/rust/mercury
WORKDIR ${APP}

EXPOSE 5001
RUN apt-get update && apt-get install -y cron ca-certificates
RUN echo "*/15 * * * * cd ${APP}; ./weatherbit" | crontab
RUN /etc/init.d/cron start
COPY --from=builder ${APP}/target/x86_64-unknown-linux-musl/release/mercury ./mercury
COPY --from=builder ${APP}/target/x86_64-unknown-linux-musl/release/weatherbit ./weatherbit
COPY --from=builder ${APP}/templates ./templates
COPY --from=builder ${APP}/static ./static
COPY --from=builder ${APP}/settings.yaml .

ENTRYPOINT ["./mercury"]
