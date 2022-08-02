FROM rust:1.61-bullseye AS build-env

WORKDIR /usr/src/app
COPY . .

ENV RUSTFLAGS '-C target-feature=+crt-static'
# Static linking requires to specify a target explicitly
# (see https://github.com/rust-lang/rust/issues/78210).
RUN export TARGET=$(rustup target list | grep -i installed | tr ' ' '\n' | head -1) && \
    cargo build \
    --target $TARGET \
    --release

# An actual image

FROM scratch

COPY --chown=1000:1000 --from=build-env /usr/src/app/target/*/release/decoyssh .

USER 1000

ENV DECOYSSH_PORT 2222
ENV DECOYSSH_IPV4_ADDR 0.0.0.0:${DECOYSSH_PORT}

EXPOSE ${DECOYSSH_PORT}/tcp

ENTRYPOINT ["/decoyssh"]
