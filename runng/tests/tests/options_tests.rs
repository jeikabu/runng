use crate::common::get_url;
use runng::{
    factory::latest::ProtocolFactory,
    options::{GetOpts, NngOption},
    socket::*,
};

#[test]
fn string_equality() -> runng::Result<()> {
    let url = get_url();
    let factory = ProtocolFactory::default();
    let rep = factory.replier_open()?.listen(&url)?;
    let sockname0 = rep.socket().get_string(NngOption::SOCKNAME)?;
    let sockname1 = rep.socket().get_string(NngOption::SOCKNAME)?;
    assert_eq!(sockname0, sockname1);
    Ok(())
}

#[test]
fn names() -> runng::Result<()> {
    assert_eq!(NngOption::SOCKNAME, NngOption::SOCKNAME);
    assert_ne!(NngOption::SOCKNAME, NngOption::PROTONAME);
    Ok(())
}
