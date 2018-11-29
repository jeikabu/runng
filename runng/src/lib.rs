pub mod aio;
pub mod ctx;
pub mod factory;
pub mod msg;
pub mod protocol;
pub mod result;
pub mod socket;
pub mod transport;

pub use self::aio::*;
pub use self::ctx::*;
pub use self::factory::*;
pub use self::result::*;
pub use self::socket::*;

extern crate futures;
extern crate runng_sys;

#[macro_use]
extern crate log;

use runng_sys::*;

// Trait where type exposes a socket, but this shouldn't be part of public API
trait RawSocket {
    fn socket(&self) -> &NngSocket;
    unsafe fn nng_socket(&self) -> nng_socket {
        self.socket().nng_socket()
    }
}