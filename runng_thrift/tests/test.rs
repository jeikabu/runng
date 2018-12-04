extern crate runng;
extern crate runng_thrift;
extern crate thrift;
extern crate ordered_float;
extern crate try_from;

#[macro_use]
extern crate log;

use runng_thrift::*;

mod test_service;

#[cfg(test)]
mod tests {
    use super::*;
    use runng::{
        *,
        msg::NngMsg,
        protocol::Subscribe,
    };
    use thrift::{
        protocol::{
            TMultiplexedOutputProtocol,
        },
        server::{
            TMultiplexedProcessor,
        }
    };

    #[test]
    fn it_works() -> NngReturn {
        let url = "inproc://test";
        let factory = runng::Latest::new();
        let publisher = factory.publisher_open()?.listen(url)?;
        let subscriber = factory.subscriber_open()?.dial(url)?;
        let topic: Vec<u8> = vec![0];
        subscriber.subscribe(&topic);
        let mut msg = NngMsg::new()?;
        msg.append_u32(0)?;
        publisher.send(msg)?;
        subscriber.recv()?;

        Ok(())
    }

    use std::{
        thread,
        sync::Arc,
    };
    use thrift::{
        server::{TProcessor},
        transport::{TIoChannel},
    }
    ;
    #[derive(Debug)]
    pub struct TServer<PRC>
    where
        PRC: TProcessor + Send + Sync,// + 'static,
    {
        processor: Arc<PRC>
    }

    impl<PRC> TServer<PRC>
    where
        PRC: TProcessor + Send + Sync,
    {
        pub fn new(processor: PRC) -> TServer<PRC> {
            TServer {
                processor: Arc::new(processor)
            }
        }
    }
    use test_service::{
        TestServiceSyncHandler,
        TestServiceSyncProcessor,
    };
    struct Handler;
    impl TestServiceSyncHandler for Handler {
        fn handle_test(&self) -> thrift::Result<bool> {
            debug!("HANDLED!");
            Ok(true)
        }
    }

    #[test]
    fn basic_thrift_works() -> NngReturn {
        let url = "inproc://test2";
        let factory = runng::Latest::new();

        let replier = factory.replier_open()?.listen(url)?;

        thread::spawn(move || {
            let mut channel = TNngChannel::new(replier.clone_socket()).unwrap();
            let (readable, writable) = channel.split().unwrap();
            let mut in_proto = TNngInputProtocol::new(readable);
            let mut out_proto = TNngOutputProtocol::new(writable);
            let handler = Handler{};
            let processor = TestServiceSyncProcessor::new(handler);

            processor.process(&mut in_proto, &mut out_proto).unwrap();
            debug!("Server done!");
        });

        let requester = factory.requester_open()?.dial(url)?;

        let mut channel = TNngChannel::new(requester.clone_socket()).unwrap();
        let (readable, writable) = channel.split().unwrap();
        let in_proto = TNngInputProtocol::new(readable);
        let out_proto = TNngOutputProtocol::new(writable);

        use test_service::TTestServiceSyncClient;
        let mut client = test_service::TestServiceSyncClient::new(in_proto, out_proto);
        client.test().unwrap();

        Ok(())
    }
    
    //#[test]
    fn thrift_works() -> NngReturn {
        let url = "inproc://test3";
        let serviceName = "blah";
        let factory = runng::Latest::new();

        let replier = factory.replier_open()?.listen(url)?;
        let mut channel = TNngChannel::new(replier.clone_socket())?;
        let (readable, writable) = channel.split().unwrap();
        let in_proto = TNngInputProtocol::new(readable);
        let out_proto = TNngOutputProtocol::new(writable);
        let mut muxer = TMultiplexedProcessor::new();
        let handler = Handler{};
        let processor = TestServiceSyncProcessor::new(handler);
        muxer.register(serviceName, Box::new(processor), false);

        thread::spawn(move || {
            
        });

        let requester = factory.requester_open()?.dial(url)?;

        let mut channel = TNngChannel::new(requester.clone_socket()).unwrap();
        let (readable, writable) = channel.split().unwrap();
        let in_proto = TNngInputProtocol::new(readable);
        let out_proto = TNngOutputProtocol::new(writable);
        let out_proto = TMultiplexedOutputProtocol::new(serviceName, out_proto);

        use test_service::TTestServiceSyncClient;
        let mut client = test_service::TestServiceSyncClient::new(in_proto, out_proto);
        client.test().unwrap();

        Ok(())
    }
}
