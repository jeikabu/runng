# jeikabu/debian-rust:arm64v8-stretch-1.32.0
# Rust on ARM64

FROM multiarch/debian-debootstrap:arm64-stretch

RUN apt-get update && apt-get install -y \
    build-essential \
    ca-certificates \
    clang \
    cmake \
    curl

ARG RUST_VER=1.32.0

# Make sure rustup and cargo are in PATH
ENV PATH "~/.cargo/bin:$PATH"
# Install rustup, skip latest toolchain and get a specific version
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain none && \
    ~/.cargo/bin/rustup default $RUST_VER
