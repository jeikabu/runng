//#![cfg(feature = "stats")]

mod common;

#[cfg(test)]
mod tests {

use log::{debug};
use runng::{
    *,
    stats::NngStat,
    stats::NngStatRoot,
};

#[test]
fn stats_example() -> NngReturn {
    // https://github.com/nanomsg/nng/issues/841
    let url = "inproc://test";
    let factory = Latest::default();
    let _pusher = factory.pusher_open()?.listen(&url)?;
    let _puller = factory.puller_open()?.dial(&url)?;

    let stats = NngStatRoot::new()?;
    let child = stats.child().unwrap();
    for stat in child.iter() {
        debug!("{}", stat.name().unwrap());
    }
    Ok(())
}

use crate::common::get_url;

}