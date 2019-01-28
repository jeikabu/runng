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

    let target_os_arch = format!(
        "{}-{}.rs",
        env::var("CARGO_CFG_TARGET_OS").unwrap(),
        env::var("CARGO_CFG_TARGET_ARCH").unwrap()
    );
    let bindings_subdir = "bindings";
    let in_path = PathBuf::from(bindings_subdir).join(target_os_arch);
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
    if cfg!(feature = "regen_bindings") {
        // https://rust-lang-nursery.github.io/rust-bindgen
        // https://docs.rs/bindgen
        let bindings = bindgen::Builder::default()
            .header("wrapper.h")
            // This is needed if use `#include <nng.h>` instead of `#include "path/nng.h"`
            //.clang_arg("-Inng/src/")
            .generate()
            .expect("Unable to generate bindings");
        std::fs::create_dir_all(bindings_subdir.to_owned())
            .expect("Unable to create binding subdirectory");
        bindings
            .write_to_file(in_path.to_owned())
            .expect(&format!("Couldn't write {:?}", in_path));
    }
    std::fs::copy(in_path.to_owned(), out_path.to_owned())
        .expect(&format!("Unable to copy {:?} to {:?}", in_path, out_path));
}
