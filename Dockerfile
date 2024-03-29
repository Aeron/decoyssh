FROM docker.io/library/rust:bullseye AS build-env

WORKDIR /usr/src/app
COPY . .

ENV CARGO_NET_GIT_FETCH_WITH_CLI true
ENV RUSTFLAGS '-C target-feature=+crt-static'

# Static linking requires to specify a target explicitly
# (see https://github.com/rust-lang/rust/issues/78210).
RUN cargo build \
    --target $(rustup target list | grep -i installed | tr ' ' '\n' | head -1) \
    --release

# An actual image

FROM scratch

LABEL org.opencontainers.image.source https://github.com/aeron/decoyssh
LABEL org.opencontainers.image.licenses ISC

COPY --from=build-env /usr/src/app/target/*/release/decoyssh .

ENV DECOYSSH_PORT 2222
ENV DECOYSSH_IPV4_ADDR 0.0.0.0:${DECOYSSH_PORT}

EXPOSE ${DECOYSSH_PORT}/tcp

ENTRYPOINT ["/decoyssh"]
