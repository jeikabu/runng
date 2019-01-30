use super::*;

use runng::{
    msg::NngMsg,
    socket::{NngSocket, RecvMsg, SendMsg},
};

use std;
use std::{
    io,
    io::{Error, ErrorKind},
};
use thrift::transport::{ReadHalf, TIoChannel, WriteHalf};

pub struct TNngChannel {
    message: NngMsg,
    socket: NngSocket,
}

impl TNngChannel {
    pub fn new(socket: NngSocket) -> runng::NngResult<TNngChannel> {
        let message = NngMsg::create()?;
        Ok(TNngChannel { message, socket })
    }

    fn helper(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let size_of_buf = std::mem::size_of_val(buf);
        buf.copy_from_slice(&self.message.body()[..size_of_buf]);
        self.message.trim(size_of_buf).unwrap();
        Ok(size_of_buf)
    }
}

impl TIoChannel for TNngChannel {
    fn split(self) -> thrift::Result<(ReadHalf<Self>, WriteHalf<Self>)>
    where
        Self: Sized,
    {
        let clone =
            unsafe { result_wrapper(TNngChannel::new(NngSocket::new(self.socket.nng_socket())))? };
        Ok((ReadHalf::new(self), WriteHalf::new(clone)))
    }
}

impl Read for TNngChannel {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        //assert!(self.message.len() == 0);
        if self.message.len() == 0 {
            let res = self.socket.recv();
            match res {
                Ok(msg) => {
                    trace!("Recv: {}", msg.len());
                    self.message = msg;
                    self.helper(buf)
                }
                Err(_) => Err(Error::from(ErrorKind::Other)),
            }
        } else {
            self.helper(buf)
        }
    }
}

impl Write for TNngChannel {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = buf.len();
        let res = self.message.append_slice(buf);
        trace!("Write {}", len);
        if let Err(_) = res {
            Err(Error::from(ErrorKind::Other))
        } else {
            Ok(len)
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        trace!("Flush {}", self.message.len());
        let mut msg = NngMsg::create()?;
        std::mem::swap(&mut self.message, &mut msg);
        self.socket.send(msg)?;
        Ok(())
    }
}
