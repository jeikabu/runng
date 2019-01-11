use log::debug;
use runng_thrift::*;

use crate::test_service;
use runng::{msg::NngMsg, protocol::Subscribe, *};
use std::{sync::Arc, thread};
use thrift::{
    protocol::TMultiplexedOutputProtocol, server::TMultiplexedProcessor, server::TProcessor,
    transport::TIoChannel,
};

#[derive(Debug)]
pub struct TServer<PRC>
where
    PRC: TProcessor + Send + Sync, // + 'static,
{
    processor: Arc<PRC>,
}

impl<PRC> TServer<PRC>
where
    PRC: TProcessor + Send + Sync,
{
    pub fn new(processor: PRC) -> TServer<PRC> {
        TServer {
            processor: Arc::new(processor),
        }
    }
}

use crate::test_service::{
    TTestServiceSyncClient, TestServiceSyncHandler, TestServiceSyncProcessor,
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
    let factory = runng::Latest::default();

    let replier = factory.replier_open()?.listen(url)?;

    thread::spawn(move || {
        let mut channel = TNngChannel::new(replier.clone_socket()).unwrap();
        let (readable, writable) = channel.split().unwrap();
        let mut in_proto = TNngInputProtocol::new(readable);
        let mut out_proto = TNngOutputProtocol::new(writable);
        let handler = Handler {};
        let processor = TestServiceSyncProcessor::new(handler);

        processor.process(&mut in_proto, &mut out_proto).unwrap();
        debug!("Server done!");
    });

    let requester = factory.requester_open()?.dial(url)?;

    let mut channel = TNngChannel::new(requester.clone_socket()).unwrap();
    let (readable, writable) = channel.split().unwrap();
    let in_proto = TNngInputProtocol::new(readable);
    let out_proto = TNngOutputProtocol::new(writable);

    let mut client = test_service::TestServiceSyncClient::new(in_proto, out_proto);
    client.test().unwrap();

    Ok(())
}

//#[test]
fn thrift_works() -> NngReturn {
    let url = "inproc://test3";
    let service_name = "blah";
    let factory = runng::Latest::default();

    let replier = factory.replier_open()?.listen(url)?;
    let mut channel = TNngChannel::new(replier.clone_socket())?;
    let (readable, writable) = channel.split().unwrap();
    let in_proto = TNngInputProtocol::new(readable);
    let out_proto = TNngOutputProtocol::new(writable);
    let mut muxer = TMultiplexedProcessor::new();
    let handler = Handler {};
    let processor = TestServiceSyncProcessor::new(handler);
    muxer.register(service_name, Box::new(processor), false);

    thread::spawn(move || {});

    let requester = factory.requester_open()?.dial(url)?;

    let mut channel = TNngChannel::new(requester.clone_socket()).unwrap();
    let (readable, writable) = channel.split().unwrap();
    let in_proto = TNngInputProtocol::new(readable);
    let out_proto = TNngOutputProtocol::new(writable);
    let out_proto = TMultiplexedOutputProtocol::new(service_name, out_proto);

    use crate::test_service::TTestServiceSyncClient;
    let mut client = test_service::TestServiceSyncClient::new(in_proto, out_proto);
    client.test().unwrap();

    Ok(())
}
