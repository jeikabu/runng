use super::*;

/// Factory to create various NNG sockets
pub trait Factory {
    fn requester_open(&self) -> NngResult<protocol::Req0>;
    fn replier_open(&self) -> NngResult<protocol::Rep0>;
    fn publisher_open(&self) -> NngResult<protocol::Pub0>;
    fn subscriber_open(&self) -> NngResult<protocol::Sub0>;
    fn pusher_open(&self) -> NngResult<protocol::Push0>;
    fn puller_open(&self) -> NngResult<protocol::Pull0>;
}

/// The latest version of all protocols
///
/// # Examples
/// ```
/// use runng::{
///     Factory,
///     factory::Latest,
/// };
/// let factory = Latest::default();
/// let publisher = factory.publisher_open();
/// ```
pub struct Latest {}

impl Default for Latest {
    fn default() -> Latest {
        Latest {}
    }
}

impl Factory for Latest {
    fn requester_open(&self) -> NngResult<protocol::Req0> {
        protocol::Req0::open()
    }
    fn replier_open(&self) -> NngResult<protocol::Rep0> {
        protocol::Rep0::open()
    }
    fn publisher_open(&self) -> NngResult<protocol::Pub0> {
        protocol::Pub0::open()
    }
    fn subscriber_open(&self) -> NngResult<protocol::Sub0> {
        protocol::Sub0::open()
    }
    fn pusher_open(&self) -> NngResult<protocol::Push0> {
        protocol::Push0::open()
    }
    fn puller_open(&self) -> NngResult<protocol::Pull0> {
        protocol::Pull0::open()
    }
}
