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

COPY --from=build-env /usr/src/app/target/*/release/decoyssh .

ENTRYPOINT ["/decoyssh"]
