extern crate byteorder;

use self::byteorder::{BigEndian, WriteBytesExt};
use msg::msg::NngMsg;
use super::*;

#[derive(Default)]
pub struct MsgBuilder {
    header: Vec<u8>,
    body: Vec<u8>,
}

impl MsgBuilder {
    pub fn append_u32(&mut self, data: u32) -> &mut Self {
        let mut bytes = [0u8; std::mem::size_of::<u32>()];
        bytes.as_mut().write_u32::<BigEndian>(data).unwrap();
        self.append_slice(&bytes)
    }
    pub fn append_slice(&mut self, data: &[u8]) -> &mut Self {
        self.body.extend_from_slice(data);
        self
    }
    pub fn append_vec(&mut self, data: &mut Vec<u8>) -> &mut Self {
        self.body.append(data);
        self
    }
    pub fn build(&self) -> NngResult<NngMsg> {
        let mut msg = NngMsg::new()?;
        let len = self.header.len();
        if len > 0 {
            msg.header_append(self.header.as_ptr(), len)?;
        }
        let len = self.body.len();
        if len > 0 {
            msg.append(self.body.as_ptr(), len)?;
        }
        Ok(msg)
    }
    pub fn clean(&mut self) -> &mut Self {
        self.header.clear();
        self.body.clear();
        self
    }
}
