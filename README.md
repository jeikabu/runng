# Runng

Rust [NNG (Nanomsg-Next-Generation)](https://github.com/nanomsg/nng).

[![travis](https://travis-ci.org/jeikabu/runng.svg?branch=master)](https://travis-ci.org/jeikabu/runng)
[![appveyor](https://ci.appveyor.com/api/projects/status/0w7puh3t2g8gt4gp/branch/master?svg=true)](https://ci.appveyor.com/project/jake-ruyi/runng/branch/master)
[![codecov](https://codecov.io/gh/jeikabu/runng/branch/master/graph/badge.svg)](https://codecov.io/gh/jeikabu/runng)
[![docs.rs](https://docs.rs/runng/badge.svg)](https://docs.rs/crate/runng/)


||||
|-|-|-
| __runng-sys__ | [![runng-sys crate](https://img.shields.io/crates/v/runng-sys.svg)](https://crates.io/crates/runng-sys) | bindings to native NNG library
| __runng__ | [![runng crate](https://img.shields.io/crates/v/runng.svg)](https://crates.io/crates/runng) | high-level wrapper for NNG
| __runng_thrift__ | [![runng-thrift crate](https://img.shields.io/crates/v/runng-thrift.svg)](https://crates.io/crates/runng-thrift) | NNG as [Apache Thrift](https://github.com/apache/thrift) transport 


## Build

1. Add [cmake](https://cmake.org) to `PATH` (defaults to "Unix Makefiles" generator)
1. `cargo build`

Once [cargo issue #4942 is resolved](https://github.com/rust-lang/cargo/issues/4942) will also be able to build with `ninja` feature ([recommended by NNG](https://github.com/nanomsg/nng#requirements)).  Currently, can:
1. Add [ninja](https://ninja-build.org/) to `PATH`
1. `cd runng-sys`
1. `cargo build --features ninja`