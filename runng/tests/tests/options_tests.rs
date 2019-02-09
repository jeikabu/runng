use crate::common::get_url;
use runng::*;
use runng_sys::*;

#[test]
fn string_equality() -> NngReturn {
    let url = get_url();
    let factory = Latest::default();
    let rep = factory.replier_open()?.listen(&url)?;
    let sockname0 = rep.socket().getopt_string(NngOption::SOCKNAME)?;
    let sockname1 = rep.socket().getopt_string(NngOption::SOCKNAME)?;
    assert_eq!(sockname0, sockname1);
    Ok(())
}

#[test]
fn names() -> NngReturn {
    assert_eq!(NngOption::SOCKNAME, NngOption::SOCKNAME);
    assert_ne!(NngOption::SOCKNAME, NngOption::PROTONAME);
    Ok(())
}
