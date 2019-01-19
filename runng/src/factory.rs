use super::*;

/// The latest version of all protocols
///
/// # Examples
/// ```
/// use runng::{
///     factory::Latest,
/// };
/// let factory = Latest::default();
/// let publisher = factory.publisher_open();
/// ```
#[derive(Default)]
pub struct Latest {}

impl Latest {
    pub fn requester_open(&self) -> NngResult<protocol::Req0> {
        protocol::Req0::open()
    }
    pub fn replier_open(&self) -> NngResult<protocol::Rep0> {
        protocol::Rep0::open()
    }
    pub fn publisher_open(&self) -> NngResult<protocol::Pub0> {
        protocol::Pub0::open()
    }
    pub fn subscriber_open(&self) -> NngResult<protocol::Sub0> {
        protocol::Sub0::open()
    }
    pub fn pusher_open(&self) -> NngResult<protocol::Push0> {
        protocol::Push0::open()
    }
    pub fn puller_open(&self) -> NngResult<protocol::Pull0> {
        protocol::Pull0::open()
    }
    pub fn pair_open(&self) -> NngResult<protocol::Pair1> {
        protocol::Pair1::open()
    }
}

/// Protocols compatible with nanomsg
#[derive(Default)]
pub struct Compat {}

impl Compat {
    pub fn requester_open(&self) -> NngResult<protocol::Req0> {
        protocol::Req0::open()
    }
    pub fn replier_open(&self) -> NngResult<protocol::Rep0> {
        protocol::Rep0::open()
    }
    pub fn publisher_open(&self) -> NngResult<protocol::Pub0> {
        protocol::Pub0::open()
    }
    pub fn subscriber_open(&self) -> NngResult<protocol::Sub0> {
        protocol::Sub0::open()
    }
    pub fn pusher_open(&self) -> NngResult<protocol::Push0> {
        protocol::Push0::open()
    }
    pub fn puller_open(&self) -> NngResult<protocol::Pull0> {
        protocol::Pull0::open()
    }
    pub fn pair_open(&self) -> NngResult<protocol::Pair0> {
        protocol::Pair0::open()
    }
}
