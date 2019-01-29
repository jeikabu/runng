
# runng-sys

Rust FFI bindings to [NNG](https://github.com/nanomsg/nng) (generated with [bindgen](https://rust-lang.github.io/rust-bindgen/introduction.html)):

> NNG, like its predecessors nanomsg (and to some extent ZeroMQ), is a lightweight, broker-less library, offering a simple API to solve common recurring messaging problems, such as publish/subscribe, RPC-style request/reply, or service discovery. The API frees the programmer from worrying about details like connection management, retries, and other common considerations, so that they can focus on the application instead of the plumbing.


## Usage

Version of this crate tracks NNG: `<NNG_version>-rc.<crate_version>` (e.g. `1.1.1-rc.2`).

To use the __latest crate__ for the most recent __stable version of NNG__ (1.1.1), in `Cargo.toml`:
```toml
runng-sys = "1.1.1-rc"
```

Requirements:
- [cmake](https://cmake.org/) in `PATH`
    - On Linux/macOS: default generator is "Unix Makefiles" and should _just work_
    - On Windows: default generator is [ninja](https://ninja-build.org/) and must also be in `PATH`
- [libclang](https://rust-lang.github.io/rust-bindgen/requirements.html)

For a more ergonomic API to NNG see [runng](https://crates.io/crates/runng).

## Features

- `cmake-ninja`: use cmake generator "Ninja"
- `cmake-vs2017`: use cmake generator "Visual Studio 15 2017"
- `cmake-vs2017-win64`: use cmake generator "Visual Studio 15 2017 Win64"
- `nng-stats`: enable NNG stats `NNG_ENABLE_STATS` (enabled by default)
- `nng-tls`: enable TLS `NNG_ENABLE_TLS` (requires mbedTLS, disabled by default)

For example, to disable stats and use Ninja cmake generator:
```toml
[dependencies.runng-sys]
version = "1.1.1-rc"
default-features = false
features = ["cmake-ninja"]
```

## Examples
```rust
use runng_sys::*;
use std::{ffi::CString, ptr::null_mut};

#[test]
fn example() {
    unsafe {
        let url = CString::new("inproc://test").unwrap();
        let url = url.as_bytes_with_nul().as_ptr() as *const std::os::raw::c_char;

        // Reply socket
        let mut rep_socket = nng_socket { id: 0 };
        nng_rep0_open(&mut rep_socket);
        nng_listen(rep_socket, url, null_mut(), 0);

        // Request socket
        let mut req_socket = nng_socket { id: 0 };
        nng_req0_open(&mut req_socket);
        nng_dial(req_socket, url, null_mut(), 0);

        // Send message
        let mut req_msg: *mut nng_msg = null_mut();
        nng_msg_alloc(&mut req_msg, 0);
        // Add a value to the body of the message
        let val = 0x12345678;
        nng_msg_append_u32(req_msg, val);
        nng_sendmsg(req_socket, req_msg, 0);

        // Receive it
        let mut recv_msg: *mut nng_msg = null_mut();
        nng_recvmsg(rep_socket, &mut recv_msg, 0);
        // Remove our value from the body of the received message
        let mut recv_val: u32 = 0;
        nng_msg_trim_u32(recv_msg, &mut recv_val);
        assert_eq!(val, recv_val);
        // Can't do this because nng uses network order (big-endian)
        //assert_eq!(val, *(nng_msg_body(recv_msg) as *const u32));

        nng_close(req_socket);
        nng_close(rep_socket);
    }
}
```