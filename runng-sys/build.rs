use cmake::Config;
use std::{env, path::PathBuf};

fn main() {
    // Compile-time features
    let generator = if cfg!(feature = "ninja") {
        "Ninja"
    } else if cfg!(feature = "vs2017") {
        "Visual Studio 15 2017 Win64"
    } else {
        if cfg!(target_family = "unix") {
            "Unix Makefiles"
        } else {
            "Ninja"
        }
    };
    let stats = if cfg!(feature = "stats") { "ON" } else { "OFF" };
    let tls = if cfg!(feature = "tls") { "ON" } else { "OFF" };

    // Run cmake to build nng
    let dst = Config::new("nng")
        .generator(generator)
        .define("CMAKE_BUILD_TYPE", "Release")
        .define("NNG_TESTS", "OFF")
        .define("NNG_TOOLS", "OFF")
        .define("NNG_ENABLE_STATS", stats)
        .define("NNG_ENABLE_TLS", tls)
        .build();

    // Check output of `cargo build --verbose`, should see something like:
    // -L native=/path/runng/target/debug/build/runng-sys-abc1234/out
    // That contains output from cmake
    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("lib").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("lib64").display()
    );

    // Tell rustc to use nng static library
    println!("cargo:rustc-link-lib=static=nng");

    let bindings_filename = "bindings.rs";
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap())
        .join(bindings_filename);
    if cfg!(feature = "regen_bindings") {
        // https://rust-lang-nursery.github.io/rust-bindgen
        // https://docs.rs/bindgen
        let bindings = bindgen::Builder::default()
            .header("wrapper.h")
            // This is needed if use `#include <nng.h>` instead of `#include "path/nng.h"`
            //.clang_arg("-Inng/src/")
            .generate()
            .expect("Unable to generate bindings");
        bindings
            .write_to_file(bindings_filename)
            .expect(&format!("Couldn't write {}", bindings_filename));
    }
    std::fs::copy(bindings_filename, out_path.to_owned())
        .expect(&format!("Unable to copy {} to {:?}", bindings_filename, out_path));
}
