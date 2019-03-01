# jeikabu/debian-rust:arm32v7-stretch-1.33.0
# Rust on ARM32

FROM multiarch/debian-debootstrap:armhf-stretch

RUN apt-get update && apt-get install -y \
    build-essential \
    ca-certificates \
    cmake \
    curl

ARG RUST_VER=1.33.0

# Make sure rustup and cargo are in PATH
ENV PATH "~/.cargo/bin:$PATH"
# Install rustup, skip latest toolchain and get a specific version
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain none && \
    ~/.cargo/bin/rustup default $RUST_VER
