//#![cfg(feature = "stats")]

use crate::common::init_logging;
use log::debug;
use runng::{factory::latest::ProtocolFactory, protocol, socket::*, stats::*};

fn init_stats() -> runng::Result<(protocol::push0::Push0, protocol::pull0::Pull0)> {
    init_logging();
    // FIXME: can remove this in NNG 1.1.2 or 1.2
    // https://github.com/nanomsg/nng/issues/841
    let url = "inproc://test";
    let factory = ProtocolFactory::default();
    let pusher = factory.pusher_open()?.listen(&url)?;
    let puller = factory.puller_open()?.dial(&url)?;
    Ok((pusher, puller))
}

#[test]
fn stats_example() -> runng::Result<()> {
    let (_pusher, _puller) = init_stats()?;

    let stats = NngStatRoot::create()?;
    let child = stats.child().unwrap();
    for stat in child.iter() {
        debug!("{}", stat.name().unwrap());
    }
    Ok(())
}
