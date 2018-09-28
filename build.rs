extern crate bindgen;
extern crate cmake;

use cmake::Config;
use std::{
    env,
    path::PathBuf,
};

fn main() {
    // https://docs.rs/cmake/0.1.34/cmake/
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

    // https://rust-lang-nursery.github.io/rust-bindgen
    // https://docs.rs/bindgen
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-Inng/src/")
        .generate()
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings");
}