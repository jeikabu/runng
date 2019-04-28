//! Factories for specific sets of protocols.

pub mod latest {
    use crate::*;

    /// The latest version of all protocols
    ///
    /// # Examples
    /// ```
    /// use runng::{
    ///     factory::latest::ProtocolFactory,
    /// };
    /// let factory = ProtocolFactory::default();
    /// let publisher = factory.publisher_open();
    /// ```
    #[derive(Debug, Default)]
    pub struct ProtocolFactory {}

    impl ProtocolFactory {
        pub fn requester_open(&self) -> Result<protocol::Req0> {
            protocol::Req0::open()
        }
        pub fn replier_open(&self) -> Result<protocol::Rep0> {
            protocol::Rep0::open()
        }
        pub fn publisher_open(&self) -> Result<protocol::Pub0> {
            protocol::Pub0::open()
        }
        pub fn subscriber_open(&self) -> Result<protocol::Sub0> {
            protocol::Sub0::open()
        }
        pub fn pusher_open(&self) -> Result<protocol::Push0> {
            protocol::Push0::open()
        }
        pub fn puller_open(&self) -> Result<protocol::Pull0> {
            protocol::Pull0::open()
        }
        pub fn pair_open(&self) -> Result<protocol::Pair1> {
            protocol::Pair1::open()
        }
    }
}

pub mod compat {
    use crate::*;

    /// Protocols compatible with nanomsg
    #[derive(Debug, Default)]
    pub struct ProtocolFactory {}

    impl ProtocolFactory {
        pub fn requester_open(&self) -> Result<protocol::Req0> {
            protocol::Req0::open()
        }
        pub fn replier_open(&self) -> Result<protocol::Rep0> {
            protocol::Rep0::open()
        }
        pub fn publisher_open(&self) -> Result<protocol::Pub0> {
            protocol::Pub0::open()
        }
        pub fn subscriber_open(&self) -> Result<protocol::Sub0> {
            protocol::Sub0::open()
        }
        pub fn pusher_open(&self) -> Result<protocol::Push0> {
            protocol::Push0::open()
        }
        pub fn puller_open(&self) -> Result<protocol::Pull0> {
            protocol::Pull0::open()
        }
        pub fn pair_open(&self) -> Result<protocol::Pair0> {
            protocol::Pair0::open()
        }
    }

}
