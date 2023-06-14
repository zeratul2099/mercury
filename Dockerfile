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
RUN echo "0 * * * * cd ${APP}; ./weatherbit >> /var/log/weatherbit" | crontab
RUN echo "#!/bin/sh\n/etc/init.d/cron start\ncd ${APP}\n./weatherbit\n./mercury" > ${APP}/start.sh
RUN chmod +x ${APP}/start.sh
COPY --from=builder ${APP}/target/x86_64-unknown-linux-musl/release/mercury ./mercury
COPY --from=builder ${APP}/target/x86_64-unknown-linux-musl/release/weatherbit ./weatherbit
COPY --from=builder ${APP}/templates ./templates
COPY --from=builder ${APP}/static ./static

ENTRYPOINT ["./start.sh"]

# Push image to you local registry with
# docker build -t <hostname>:5000/mercury:latest .
# docker push <hostname>:5000/mercury:latest

# Pull from registry:
# docker pull <hostname>:5000/mercury:latest

# Start container with
# docker run -dP -p5001:5001  --mount type=bind,source=/<path-to>/settings.yaml,target=/home/rust/mercury/settings.yaml --name mercury localhost:5000/mercury:latest
# or
# docker-compose up -d
