use crate::msg::NngMsg;
use crate::*;
use byteorder::{BigEndian, WriteBytesExt};

/// Build `NngMsg` using fluent API.  See [nng_msg](https://nanomsg.github.io/nng/man/v1.1.0/nng_msg.5).
///
/// # Examples
///
/// ``` -> Result<(), NngFail>
/// let msg = MsgBuilder::default()
///     .append_u32(0)
///     .build()?;
/// ```
#[derive(Default)]
pub struct MsgBuilder {
    header: Vec<u8>,
    body: Vec<u8>,
}

impl MsgBuilder {
    /// Append `u32` to the body.
    pub fn append_u32(&mut self, data: u32) -> &mut Self {
        let mut bytes = [0u8; std::mem::size_of::<u32>()];
        bytes.as_mut().write_u32::<BigEndian>(data).unwrap();
        self.append_slice(&bytes)
    }

    /// Append `u8` slice to the body.
    pub fn append_slice(&mut self, data: &[u8]) -> &mut Self {
        self.body.extend_from_slice(data);
        self
    }

    /// Append `u8` vec to the body.
    pub fn append_vec(&mut self, data: &mut Vec<u8>) -> &mut Self {
        self.body.append(data);
        self
    }

    /// Create a `NngMsg` using contents of this builder.
    pub fn build(&self) -> NngResult<NngMsg> {
        let mut msg = NngMsg::create()?;
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

    /// Clears the builder, removing all values.
    pub fn clean(&mut self) -> &mut Self {
        self.header.clear();
        self.body.clear();
        self
    }
}
