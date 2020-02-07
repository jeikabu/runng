use crate::common::*;
use runng::{
    factory::latest::ProtocolFactory,
    options::{GetOpts, NngOption},
    socket::*,
};

#[test]
fn string_equality() -> runng::Result<()> {
    let url = get_url();
    let factory = ProtocolFactory::default();
    let mut rep = factory.replier_open()?;
    rep.listen(&url)?;
    let sockname0 = rep.get_string(NngOption::SOCKNAME)?;
    let sockname1 = rep.get_string(NngOption::SOCKNAME)?;
    assert_eq!(sockname0, sockname1);
    Ok(())
}

#[test]
fn names() -> runng::Result<()> {
    assert_eq!(NngOption::SOCKNAME, NngOption::SOCKNAME);
    assert_ne!(NngOption::SOCKNAME, NngOption::PROTONAME);
    Ok(())
}

#[test]
fn sockaddr() -> runng::Result<()> {
    for url in get_urls() {
        let factory = ProtocolFactory::default();
        let sock = factory.pair_open()?;
        let listener = sock.listener_create(&url)?;
        listener.start()?;
        let sockaddr = listener.get_sockaddr(NngOption::LOCADDR)?;
        use SockAddr::*;
        match sockaddr {
            Inproc(_) => assert!(url.starts_with("inproc://")),
            Ipc(_) => assert!(url.starts_with("ipc://")),
            In(_) | In6(_) => assert!(url.starts_with("tcp://") || url.starts_with("ws://")),
            Zt(_) => assert!(url.starts_with("zt://")),
            _ => panic!(),
        }
    }
    Ok(())
}
