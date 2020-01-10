# nng_async / RuNNG

Rust [NNG (Nanomsg-Next-Generation)](https://github.com/nanomsg/nng):

> NNG, like its predecessors nanomsg (and to some extent ZeroMQ), is a lightweight, broker-less library, offering a simple API to solve common recurring messaging problems, such as publish/subscribe, RPC-style request/reply, or service discovery. The API frees the programmer from worrying about details like connection management, retries, and other common considerations, so that they can focus on the application instead of the plumbing.

[![travis](https://travis-ci.org/jeikabu/runng.svg?branch=master)](https://travis-ci.org/jeikabu/runng)
[![codecov](https://codecov.io/gh/jeikabu/runng/branch/master/graph/badge.svg)](https://codecov.io/gh/jeikabu/runng)
[![docs.rs](https://docs.rs/runng/badge.svg)](https://docs.rs/crate/runng/)


|Repository|Crate|Details|
|-|-|-
| __nng_async__ / __runng__ | [![runng crate](https://img.shields.io/crates/v/runng.svg)](https://crates.io/crates/nng_async) | high-level wrapper for NNG
| [__nng-sys__](https://github.com/jeikabu/nng-rust) | [![runng-sys crate](https://img.shields.io/crates/v/nng-sys.svg)](https://crates.io/crates/nng-sys) | bindings to native NNG library
| [__runng_examples__](https://github.com/jeikabu/runng_examples) | | Additional examples
| [__runng_thrift__](https://github.com/jeikabu/runng_thrift) | [![runng-thrift crate](https://img.shields.io/crates/v/runng-thrift.svg)](https://crates.io/crates/runng-thrift) | NNG as [Apache Thrift](https://github.com/apache/thrift) transport

## Usage

In `Cargo.toml`:
```toml
runng = "0.3"
```

Requirements:
- [cmake](https://cmake.org/) in `PATH`
    - On Linux/macOS: default generator is "Unix Makefiles" and should _just work_
    - On Windows: default generator is usually Visual Studio
- _Optional_ libclang needed if using `build-bindgen` feature to run [bindgen](https://rust-lang.github.io/rust-bindgen/requirements.html)

## Build

1. Update submodules: `git submodule update --init --recursive`
1. Install requirements
1. `cargo build`
