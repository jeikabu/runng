// Suppress the flurry of warnings caused by using "C" naming conventions
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// This matches bindgen::Builder output
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

impl nng_msg {
    pub fn new() -> nng_msg {
        nng_msg { _unused: [] }
    }
}

impl nng_aio {
    pub fn new() -> nng_aio {
        nng_aio { _unused: [] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
    }
}
