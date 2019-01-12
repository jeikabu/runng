# Runng

Rust [NNG (Nanomsg-Next-Generation)](https://github.com/nanomsg/nng):

> NNG, like its predecessors nanomsg (and to some extent ZeroMQ), is a lightweight, broker-less library, offering a simple API to solve common recurring messaging problems, such as publish/subscribe, RPC-style request/reply, or service discovery. The API frees the programmer from worrying about details like connection management, retries, and other common considerations, so that they can focus on the application instead of the plumbing.

[![travis](https://travis-ci.org/jeikabu/runng.svg?branch=master)](https://travis-ci.org/jeikabu/runng)
[![appveyor](https://ci.appveyor.com/api/projects/status/0w7puh3t2g8gt4gp/branch/master?svg=true)](https://ci.appveyor.com/project/jake-ruyi/runng/branch/master)
[![codecov](https://codecov.io/gh/jeikabu/runng/branch/master/graph/badge.svg)](https://codecov.io/gh/jeikabu/runng)
[![docs.rs](https://docs.rs/runng/badge.svg)](https://docs.rs/crate/runng/)


||||
|-|-|-
| __runng-sys__ | [![runng-sys crate](https://img.shields.io/crates/v/runng-sys.svg)](https://crates.io/crates/runng-sys) | bindings to native NNG library
| __runng__ | [![runng crate](https://img.shields.io/crates/v/runng.svg)](https://crates.io/crates/runng) | high-level wrapper for NNG
| __runng_thrift__ | [![runng-thrift crate](https://img.shields.io/crates/v/runng-thrift.svg)](https://crates.io/crates/runng-thrift) | NNG as [Apache Thrift](https://github.com/apache/thrift) transport

## Usage

In `Cargo.toml`:
```toml
runng = "0.1"
# OR
runng-sys = "1.1.1-rc"
```

## Build

1. Update submodules: `git submodule update --init --recursive`
1. Add [cmake](https://cmake.org) to `PATH`
    - On Linux/macOS: default generator is "Unix Makefiles"
    - On Windows: default generator is [ninja](https://ninja-build.org/) and must also be in `PATH`
1. `cargo build`

Once [cargo issue #4942 is resolved](https://github.com/rust-lang/cargo/issues/4942) will also be able to build with `ninja` feature ([recommended by NNG](https://github.com/nanomsg/nng#requirements)).  Currently, can:
1. Add [ninja](https://ninja-build.org/) to `PATH`
1. `cd runng-sys`
1. `cargo build --features ninja`

To build optional packages: `cargo build --all`