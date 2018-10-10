pub mod reqrep;

pub use self::reqrep::*;

use runng_sys::*;
use super::*;


type NngOpenFunc = unsafe extern "C" fn(*mut runng_sys::nng_socket_s) -> i32;

fn open<T, O, S>(open_func: O, socket_create_func: S) -> NngResult<T>
    where O: Fn(&mut nng_socket) -> i32,
        S: Fn(NngSocket) -> T
{
    let mut socket = nng_socket { id: 0 };
    let res = open_func(&mut socket);
    if res == 0 {
        let socket = NngSocket::new(socket);
        Ok(socket_create_func(socket))
    } else {
        Err(NngFail::from_i32(res))
    }
}

