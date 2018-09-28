// https://rust-lang-nursery.github.io/rust-bindgen
extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to tell rustc to link 

    // bindgen::Builder is the main entry point to bindgen, and lets
    // you build up options for the resulting bindings
    let bindings = bindgen::Builder::default()
        // Input header we generate bindings for
        .header("wrapper.h")
        // Finish the builder and generate the bindings
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings");
}