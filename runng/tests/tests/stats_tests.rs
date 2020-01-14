//#![cfg(feature = "stats")]

use crate::common::*;
use log::debug;
use runng::{factory::compat::ProtocolFactory, socket::*, stats::*};

fn init_stats() -> runng::Result<(protocol::pair0::Pair0, protocol::pair0::Pair0)> {
    init_logging();
    let url = get_url();
    let factory = ProtocolFactory::default();
    let mut p0 = factory.pair_open()?;
    p0.listen(&url)?;
    let mut p1 = factory.pair_open()?;
    p1.dial(&url)?;
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
