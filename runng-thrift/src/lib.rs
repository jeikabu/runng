extern crate thrift;
extern crate runng;

use std::{
    io,
    io::{
        prelude::*,
    }
};


mod nng_channel;
pub use nng_channel::*;

mod nng_protocol;
pub use nng_protocol::*;

enum NngThriftError {
    Nng(runng::NngFail),
    Thrift(thrift::Error),
}
type NngThriftResult<T> = Result<T, NngThriftError>;

fn ResultWrapper<T>(result: runng::NngResult<T>) -> NngThriftResult<T> {
    match result {
        Ok(result) => Ok(result),
        Err(result) => Err(NngThriftError::Nng(result)),
    }
}

impl From<NngThriftError> for thrift::Error {
    fn from(err: NngThriftError) -> thrift::Error {
        match err {
            NngThriftError::Nng(err) => {
                let err: io::Error = From::from(err);
                thrift::Error::from(err)
            }
            NngThriftError::Thrift(err) => err,
        }
    }
}

// impl From<NngThriftError> for io::Error {
//     fn from(err: NngThriftError) -> io::Error {
//         match err {
//             NngThriftError::Nng(err) => From::from(err)
//         }
//     }
// }


/// FIXME: copied from thrift source code (src/transport/mod.rs) because need access to `handle` member in order to create Read/WriteHalf
/// Remove once add `new()` associated method

use std::ops::{Deref, DerefMut};

/// The readable half of an object returned from `TIoChannel::split`.
#[derive(Debug)]
pub struct ReadHalf<C>
where
    C: Read,
{
    handle: C,
}

/// The writable half of an object returned from `TIoChannel::split`.
#[derive(Debug)]
pub struct WriteHalf<C>
where
    C: Write,
{
    handle: C,
}

impl<C> Read for ReadHalf<C>
where
    C: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.handle.read(buf)
    }
}

impl<C> Write for WriteHalf<C>
where
    C: Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.handle.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.handle.flush()
    }
}

impl<C> Deref for ReadHalf<C>
where
    C: Read,
{
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl<C> DerefMut for ReadHalf<C>
where
    C: Read,
{
    fn deref_mut(&mut self) -> &mut C {
        &mut self.handle
    }
}

impl<C> Deref for WriteHalf<C>
where
    C: Write,
{
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl<C> DerefMut for WriteHalf<C>
where
    C: Write,
{
    fn deref_mut(&mut self) -> &mut C {
        &mut self.handle
    }
}
