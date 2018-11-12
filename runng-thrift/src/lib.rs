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


#[cfg(test)]
mod tests {
    use super::*;
    use runng::{
        Dial,
        Factory,
        Listen,
        RecvMsg,
        SendMsg,
        Socket,
        msg::NngMsg,
        protocol::Subscribe,
    };
    use thrift::{
        protocol::{
            TMultiplexedOutputProtocol,
        },
        transport::TIoChannel,
    };

    #[test]
    fn it_works() {
        let factory = runng::Latest::new();
        let url = "inproc://test";
        let publisher = factory.publisher_open().unwrap().listen(url).unwrap();
        let subscriber = factory.subscriber_open().unwrap().dial(url).unwrap();
        let topic: Vec<u8> = vec![0];
        subscriber.subscribe(&topic);
        let mut msg = NngMsg::new().unwrap();
        msg.append_u32(0).unwrap();
        publisher.send(msg).unwrap();
        subscriber.recv().unwrap();
    }

    #[test]
    fn thrift_works() {
        let url = "inproc://test2";
        let factory = runng::Latest::new();
        let replier = factory.replier_open().unwrap()
            .listen(url).unwrap();
        let requester = factory.requester_open().unwrap().dial(url).unwrap();

        let mut channel = TNngChannel::new(requester.take()).unwrap();
        let (readable, writable) = channel.split().unwrap();
        let in_proto = TNngInputProtocol::new(readable);
        let out_proto = TNngOutputProtocol::new(writable);
        let out_proto = TMultiplexedOutputProtocol::new("blah", out_proto);

    }
}
