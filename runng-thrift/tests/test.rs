extern crate runng;
extern crate runng_thrift;
extern crate thrift;
extern crate ordered_float;
extern crate try_from;

use runng_thrift::*;

mod test_service;

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
        server::{
            TMultiplexedProcessor,
        }
    };

    #[test]
    fn it_works() {
        let factory = runng::Latest::new();
        let publisher = factory.publisher_open().unwrap();
        let subscriber = factory.subscriber_open().unwrap();
        let url = "inproc://test";
        publisher.listen(url).unwrap();
        subscriber.dial(url).unwrap();
        let topic: Vec<u8> = vec![0];
        subscriber.subscribe(&topic);
        let mut msg = NngMsg::new().unwrap();
        msg.append_u32(0).unwrap();
        publisher.send(msg).unwrap();
        subscriber.recv().unwrap();
    }

    use std::{
        thread,
        sync::Arc,
    };
    use thrift::server::{
        TProcessor,
    };
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
            println!("HANDLED!");
            Ok(true)
        }
    }

    #[test]
    fn basic_thrift_works() {
        let url = "inproc://test2";
        let factory = runng::Latest::new();

        let replier = factory.replier_open().unwrap();
        replier.listen(url).unwrap();
        

        thread::spawn(move || {
            let mut channel = TNngChannel::new(replier.take()).unwrap();
            let (readable, writable) = channel.split().unwrap();
            let mut in_proto = TNngInputProtocol::new(readable);
            let mut out_proto = TNngOutputProtocol::new(writable);
            let handler = Handler{};
            let processor = TestServiceSyncProcessor::new(handler);

            processor.process(&mut in_proto, &mut out_proto).unwrap();
            println!("Server done!");
        });

        let requester = factory.requester_open().unwrap();
        requester.dial(url).unwrap();

        let mut channel = TNngChannel::new(requester.take()).unwrap();
        let (readable, writable) = channel.split().unwrap();
        let in_proto = TNngInputProtocol::new(readable);
        let out_proto = TNngOutputProtocol::new(writable);

        use test_service::TTestServiceSyncClient;
        let mut client = test_service::TestServiceSyncClient::new(in_proto, out_proto);
        client.test().unwrap();
    }
    
    #[test]
    fn thrift_works() {
        let url = "inproc://test3";
        let serviceName = "blah";
        let factory = runng::Latest::new();

        

        let replier = factory.replier_open().unwrap();
        replier.listen(url).unwrap();
        let mut channel = TNngChannel::new(replier.take()).unwrap();
        let (readable, writable) = channel.split().unwrap();
        let in_proto = TNngInputProtocol::new(readable);
        let out_proto = TNngOutputProtocol::new(writable);
        let mut muxer = TMultiplexedProcessor::new();
        let handler = Handler{};
        let processor = TestServiceSyncProcessor::new(handler);
        muxer.register(serviceName, Box::new(processor), false);

        thread::spawn(move || {
            
        });

        let requester = factory.requester_open().unwrap();
        requester.dial(url).unwrap();

        let mut channel = TNngChannel::new(requester.take()).unwrap();
        let (readable, writable) = channel.split().unwrap();
        let in_proto = TNngInputProtocol::new(readable);
        let out_proto = TNngOutputProtocol::new(writable);
        let out_proto = TMultiplexedOutputProtocol::new(serviceName, out_proto);

        use test_service::TTestServiceSyncClient;
        let mut client = test_service::TestServiceSyncClient::new(in_proto, out_proto);
        client.test().unwrap();
    }
}
