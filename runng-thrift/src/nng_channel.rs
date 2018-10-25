extern crate runng;
extern crate thrift;

use super::*;

use runng::{
    msg::{
        NngMsg
    },
    socket::{
        NngSocket,
        RecvMsg,
        SendMsg,
    },
};

use std;
use std::{
    io,
    io::{
        {Error, ErrorKind},
    }
};
// use thrift::transport::{
//     TIoChannel,
//     ReadHalf,
//     WriteHalf,
// };

pub struct TNngChannel {
    message: NngMsg,
    socket: NngSocket,
}

impl TNngChannel {
    pub fn new(socket: NngSocket) -> runng::NngResult<TNngChannel> {
        let message = NngMsg::new()?;
        Ok(TNngChannel {
            message,
            socket,
        })
    }

    // FIXME: this should be in `impl TIoChannel`, but cannot construct Read/WriteHalf defined in thrift crate
    pub fn split(self) -> thrift::Result<(ReadHalf<Self>, WriteHalf<Self>)>
    where
        Self: Sized,
    {
        let clone = unsafe {
            ResultWrapper(TNngChannel::new(NngSocket::new(self.socket.nng_socket())))?
        };
        Ok((
            ReadHalf { handle: self },
            WriteHalf { handle: clone }
        ))
    }
}

// impl TIoChannel for TNngChannel {
    
// }

impl Read for TNngChannel {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        assert!(self.message.len() == 0);
        let res = self.socket.recv();
        match res {
            Ok(msg) => {
                // let len = msg.len();
                // let body = msg.body();
                // buf.copy_from_slice(body);
                Ok(0)
            },
            Err(_) => Err(Error::from(ErrorKind::Other))
        }
    }
}

impl Write for TNngChannel {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = buf.len();
        let res = self.message.append(buf.as_ptr(), len);
        if let Err(_) = res {
            Err(Error::from(ErrorKind::Other))
        } else {
            Ok(len)
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        let mut msg = NngMsg::new()?;
        std::mem::swap(&mut self.message, &mut msg);
        self.socket.send(msg)?;
        Ok(())
    }
}