pub mod aio;
pub mod ctx;
pub mod factory;
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

use runng_sys::*;

// public interface ISocketFactory
//     {
//         INngResult<IBusSocket> BusOpen();
//         INngResult<IReqSocket> RequesterOpen();
//         INngResult<IRepSocket> ReplierOpen();
//         INngResult<IPubSocket> PublisherOpen();
//         INngResult<ISubSocket> SubscriberOpen();
//         INngResult<IPushSocket> PusherOpen();
//         INngResult<IPullSocket> PullerOpen();
        
//         IListener ListenerCreate(ISocket socket, string url);
//         IDialer DialerCreate(ISocket socket, string url);

//         INngResult<TSocket> Dial<TSocket>(INngResult<TSocket> socketRes, string url) where TSocket : ISocket;
//         INngResult<TSocket> Listen<TSocket>(INngResult<TSocket> socketRes, string url) where TSocket : ISocket;
//     }



// Trait where type exposes a socket, but this shouldn't be part of public API
trait RawSocket {
    fn socket(&self) -> nng_socket;
}