use log::trace;
use std::{io, io::prelude::*};

mod nng_channel;
pub use crate::nng_channel::*;

mod nng_protocol;
pub use crate::nng_protocol::*;

enum NngThriftError {
    Nng(runng::NngFail),
    Thrift(thrift::Error),
}
type NngThriftResult<T> = Result<T, NngThriftError>;

fn result_wrapper<T>(result: runng::NngResult<T>) -> NngThriftResult<T> {
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
