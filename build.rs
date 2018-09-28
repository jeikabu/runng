// https://docs.rs/cmake/0.1.34/cmake/
extern crate cmake;

use cmake::Config;
use std::env;

fn main() {
    let dst = Config::new("nng")
        .generator("Ninja")
        .define("CMAKE_BUILD_TYPE", "Release")
        .define("NNG_TESTS", "OFF")
        .define("NNG_TOOLS", "OFF")
        .build();
    
    // Check output of `cargo build --verbose`, should see something like:
    // -L native=/path/runng/target/debug/build/runng-sys-abc1234/out
    // That contains output from cmake
    println!("cargo:rustc-link-search=native={}", dst.join("lib").display());
    println!("cargo:rustc-link-lib=static=nng");
}