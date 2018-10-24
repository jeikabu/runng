extern crate thrift;
extern crate runng;

use runng::socket::{RecvMsg, SendMsg};

use std::{
    io,
    io::{
        {Error, ErrorKind},
        prelude::*,
    }
};

pub struct TNngChannel {
    message: runng::msg::NngMsg,
    socket: runng::socket::NngSocket,
}

impl TNngChannel {
    pub fn new(socket: runng::socket::NngSocket) -> runng::NngResult<TNngChannel> {
        let message = runng::msg::NngMsg::new()?; 
        Ok(TNngChannel {
            message,
            socket,
        })
    }
}

impl Read for TNngChannel {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        assert!(self.message.len() == 0);
        let res = self.socket.recv();
        match res {
            Ok(msg) => {
                self.message = msg;
                Ok(self.message.len())
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
        let mut msg = runng::msg::NngMsg::new()?;
        std::mem::swap(&mut self.message, &mut msg);
        self.socket.send(msg)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use runng::{
        Dial,
        Factory,
        Listen,
        msg::NngMsg,
        protocol::Subscribe,
    };

    #[test]
    fn it_works() {
        let factory = runng::Latest::new();
        let publisher = factory.publisher_open().unwrap();
        let subscriber = factory.subscriber_open().unwrap();
        let url = "inproc://test";
        publisher.listen(url).unwrap();
        subscriber.dial(url).unwrap();
        let topic: Vec<u8> = vec![0];
        subscriber.subscribe(&topic);
        let mut msg = NngMsg::new().unwrap();
        msg.append_u32(0).unwrap();
        publisher.send(msg).unwrap();
        subscriber.recv().unwrap();
    }
}
