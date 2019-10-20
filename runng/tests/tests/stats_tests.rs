//#![cfg(feature = "stats")]

use crate::common::*;
use log::debug;
use runng::{factory::compat::ProtocolFactory, socket::*, stats::*};

fn init_stats() -> runng::Result<(protocol::pair0::Pair0, protocol::pair0::Pair0)> {
    init_logging();
    // FIXME: can remove this in NNG 1.1.2 or 1.2
    // https://github.com/nanomsg/nng/issues/841
    let url = get_url();
    let factory = ProtocolFactory::default();
    let p0 = factory.pair_open()?.listen(&url)?;
    let p1 = factory.pair_open()?.dial(&url)?;
    Ok((p0, p1))
}

#[test]
fn stats_example() -> runng::Result<()> {
    let (_p0, _p1) = init_stats()?;
    // NB: without a short sleep there's ~10% chance we get stuck reading socket stats
    sleep_brief();
    let stats = NngStatRoot::new()?;
    let child = stats.child().unwrap();
    for stat in child.iter() {
        debug!("{}", stat.name().unwrap());
    }
    Ok(())
}
