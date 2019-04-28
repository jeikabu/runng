//! Byte streams.

use crate::{asyncio::*, *};
use futures::sync::oneshot;
use log::trace;
use runng_derive::{NngGetOpts, NngSetOpts};
use runng_sys::*;
use std::ptr;

/// Byte stream corresponding to TCP, UNIX domain socket, named pipe, etc. connection.
/// Wraps `nng_stream`
#[derive(Debug, NngGetOpts, NngSetOpts)]
#[prefix = "nng_stream_"]
pub struct NngStream {
    #[nng_member]
    stream: *mut nng_stream,
}

/// List of scather/gather bytes for vectored I/O.
/// Will be replaced with std iovec once that stabilizes:
/// https://github.com/jeikabu/runng/issues/47
pub type IoVec = Vec<Vec<u8>>;

fn as_iov(vec: &IoVec) -> Vec<nng_iov> {
    let mut iovs = Vec::new();
    for buffer in vec.iter() {
        let iov = nng_iov {
            iov_buf: buffer.as_ptr() as *mut _,
            iov_len: buffer.len(),
        };
        iovs.push(iov);
    }
    iovs
}

impl NngStream {
    /// Send to byte stream.
    pub fn send(
        &mut self,
        queue: &mut impl AioWorkQueue,
        iov: IoVec,
    ) -> oneshot::Receiver<Result<usize>> {
        let (sender, receiver) = oneshot::channel();
        let send = SendAioWork(self.stream, iov, Some(sender));
        let send = Box::new(send);
        queue.push_back(send);
        receiver
    }

    /// Receive from byte stream.
    pub fn recv(
        &mut self,
        queue: &mut impl AioWorkQueue,
        iov: IoVec,
    ) -> oneshot::Receiver<Result<IoVec>> {
        let (sender, receiver) = oneshot::channel();
        let recv = RecvAioWork(self.stream, iov, Some(sender));
        let recv = Box::new(recv);
        queue.push_back(recv);
        receiver
    }

    /// Close the stream.
    pub fn close(&self) {
        unsafe { nng_stream_close(self.stream) }
    }
}

impl Drop for NngStream {
    /// Close stream and release resources.
    fn drop(&mut self) {
        unsafe { nng_stream_free(self.stream) }
    }
}

struct SendAioWork(
    *mut nng_stream,
    IoVec,
    Option<oneshot::Sender<Result<usize>>>,
);

impl AioWork for SendAioWork {
    fn begin(&self, aio: &NngAio) {
        trace!("Sending...");
        unsafe {
            let iovs = as_iov(&self.1);
            aio.set_iov(&iovs).unwrap();
            nng_stream_send(self.0, aio.nng_aio());
        }
    }
    fn finish(&mut self, aio: &NngAio) {
        unsafe {
            let res = aio.result();
            trace!("Send: {:?}", res);
            let res = match res {
                Ok(()) => Ok(aio.aio_count()),
                Err(err) => Err(err),
            };
            if let Err(_) = self.2.take().unwrap().send(res) {
                debug!("Finish failed");
            }
        }
    }
}

struct RecvAioWork(
    *mut nng_stream,
    IoVec,
    Option<oneshot::Sender<Result<IoVec>>>,
);

impl AioWork for RecvAioWork {
    fn begin(&self, aio: &NngAio) {
        trace!("Receiving...");
        unsafe {
            let iovs = as_iov(&self.1);
            aio.set_iov(&iovs).unwrap();
            nng_stream_recv(self.0, aio.nng_aio());
        }
    }
    fn finish(&mut self, aio: &NngAio) {
        unsafe {
            let res = aio.result();
            trace!("Receive: {:?}", res);
            let res = res.map(|_| self.1.to_owned());
            if let Err(_) = self.2.take().unwrap().send(res) {
                debug!("Finish failed");
            }
        }
    }
}

/// Byte stream listener.
/// Wraps `nng_stream_listener`
#[derive(Debug, NngGetOpts, NngSetOpts)]
#[prefix = "nng_stream_listener_"]
pub struct StreamListener {
    #[nng_member]
    listener: *mut nng_stream_listener,
}

impl StreamListener {
    /// Allocate byte stream listener.
    pub fn alloc(addr: &str) -> Result<Self> {
        let mut listener: *mut nng_stream_listener = ptr::null_mut();
        let res = unsafe {
            let (_cstring, addr) = to_cstr(addr)?;
            let res = nng_stream_listener_alloc(&mut listener, addr);
            nng_int_to_result(res)
        };
        res.map(|_| Self { listener })
    }

    /// Allocate byte stream listener.
    pub fn alloc_url(url: nng_url) -> Result<Self> {
        unimplemented!()
    }

    /// Bind listener to address.
    pub fn listen(&self) -> Result<()> {
        unsafe {
            let res = nng_stream_listener_listen(self.listener);
            nng_int_to_result(res)
        }
    }

    /// Accept incoming connection from [dialer].
    ///
    /// [dialer]: struct.StreamDialer.html
    pub fn accept(
        &mut self,
        queue: &mut impl AioWorkQueue,
    ) -> oneshot::Receiver<Result<NngStream>> {
        let (sender, receiver) = oneshot::channel();
        let accept = AcceptAioWork(self.listener, Some(sender));
        let accept = Box::new(accept);
        queue.push_back(accept);
        receiver
    }

    /// Close the stream.
    pub fn close(&self) {
        unsafe { nng_stream_listener_close(self.listener) }
    }
}

impl Drop for StreamListener {
    /// Close listener and release resources.
    fn drop(&mut self) {
        unsafe { nng_stream_listener_free(self.listener) }
    }
}

struct AcceptAioWork(
    *mut nng_stream_listener,
    Option<oneshot::Sender<Result<NngStream>>>,
);

impl AioWork for AcceptAioWork {
    fn begin(&self, aio: &NngAio) {
        trace!("Accepting...");
        unsafe {
            nng_stream_listener_accept(self.0, aio.nng_aio());
        }
    }
    fn finish(&mut self, aio: &NngAio) {
        unsafe {
            let res = aio.result();
            trace!("Accept: {:?}", res);
            let res = match res {
                Ok(()) => {
                    let ptr = aio.get_output(0);
                    let stream = NngStream {
                        stream: ptr as *mut nng_stream,
                    };
                    Ok(stream)
                }
                Err(err) => Err(err),
            };
            if let Err(_) = self.1.take().unwrap().send(res) {
                debug!("Finish failed");
            }
        }
    }
}

/// Byte stream dialer.  Wraps `nng_stream_dialer`
#[derive(Debug, NngGetOpts, NngSetOpts)]
#[prefix = "nng_stream_dialer_"]
pub struct StreamDialer {
    #[nng_member]
    dialer: *mut nng_stream_dialer,
}

impl StreamDialer {
    /// Allocate byte stream dialer.
    pub fn alloc(addr: &str) -> Result<Self> {
        let mut dialer: *mut nng_stream_dialer = ptr::null_mut();
        let res = unsafe {
            let (_cstring, addr) = to_cstr(addr)?;
            let res = nng_stream_dialer_alloc(&mut dialer, addr);
            nng_int_to_result(res)
        };
        res.map(|_| Self { dialer })
    }

    pub fn alloc_url(url: nng_url) -> Result<Self> {
        unimplemented!()
    }

    /// Initiate outgoing connection to [listener].
    ///
    /// [listener]: struct.StreamListener.html
    pub fn dial(&mut self, queue: &mut impl AioWorkQueue) -> oneshot::Receiver<Result<NngStream>> {
        let (sender, receiver) = oneshot::channel();
        let accept = DialAioWork(self.dialer, Some(sender));
        let accept = Box::new(accept);
        queue.push_back(accept);
        receiver
    }

    /// Close the stream.
    pub fn close(&self) {
        unsafe { nng_stream_dialer_close(self.dialer) }
    }
}

impl Drop for StreamDialer {
    /// Close dialer and release resources.
    fn drop(&mut self) {
        unsafe { nng_stream_dialer_free(self.dialer) }
    }
}

struct DialAioWork(
    *mut nng_stream_dialer,
    Option<oneshot::Sender<Result<NngStream>>>,
);

impl AioWork for DialAioWork {
    fn begin(&self, aio: &NngAio) {
        unsafe {
            nng_stream_dialer_dial(self.0, aio.nng_aio());
        }
    }
    fn finish(&mut self, aio: &NngAio) {
        unsafe {
            let res = aio.result();
            let res = match res {
                Ok(()) => {
                    let ptr = aio.get_output(0);
                    let stream = NngStream {
                        stream: ptr as *mut nng_stream,
                    };
                    Ok(stream)
                }
                Err(err) => Err(err),
            };
            if let Err(_) = self.1.take().unwrap().send(res) {
                debug!("Finish failed");
            }
        }
    }
}
