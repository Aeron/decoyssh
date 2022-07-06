FROM rust:1.61-bullseye AS build-env

ENV DEBIAN_FRONTEND noninteractive

RUN apt-get update -qq && apt-get install -y --no-install-recommends \
    musl-tools \
    musl-dev
RUN rm -r /var/lib/apt/lists /var/cache/apt/archives
RUN update-ca-certificates

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/app
COPY . .

ENV RUSTFLAGS '-C target-feature=+crt-static'
RUN cargo build \
    --target x86_64-unknown-linux-musl \
    --release

# An actual image

FROM scratch

COPY --from=build-env /usr/src/app/target/x86_64-unknown-linux-musl/release/decoyssh .

ENTRYPOINT ["/decoyssh"]
