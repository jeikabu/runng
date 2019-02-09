use crate::common::rand_msg;
use rand::Rng;
use runng::*;

#[test]
fn equality() -> NngReturn {
    // Cloned messages are equal
    let msg = rand_msg()?;
    let dupe = msg.clone();
    assert_eq!(msg, dupe);

    // Different body are not equal
    {
        let mut other = msg::NngMsg::with_size(128)?;
        rand::thread_rng().fill(other.as_mut_slice());
        assert_ne!(msg, other);
    }

    // Different header are not equal
    {
        let mut msg = rand_msg()?;
        let mut dupe = msg.clone();

        let header_bytes = vec![0, 1, 2, 3];
        dupe.header_append_slice(header_bytes.as_slice())?;
        assert_ne!(msg, dupe);

        // Same header are
        msg.header_append_slice(header_bytes.as_slice())?;
        assert_eq!(msg, dupe);
    }

    // Empty messages are equal
    {
        let empty0 = msg::NngMsg::create()?;
        let empty1 = msg::NngMsg::create()?;
        assert_eq!(empty0, empty1);
    }
    Ok(())
}
