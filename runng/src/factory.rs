use super::*;

pub trait Factory {
    fn requester_open(&self) -> NngResult<protocol::Req0>;
    fn replier_open(&self) -> NngResult<protocol::Rep0>;
}

pub struct Latest {
}

impl Latest {
    pub fn new() -> Latest {
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
}