//! NNG options.

use crate::{mem::NngString, *};
use std::{os::raw::c_char, time};

/// Types which support getting NNG options.
/// Also see [`SetOpts`](trait.SetOpts.html).
pub trait GetOpts {
    fn get_bool(&self, option: NngOption) -> Result<bool>;
    fn get_int(&self, option: NngOption) -> Result<i32>;
    fn get_ms(&self, option: NngOption) -> Result<nng_duration>;
    fn get_size(&self, option: NngOption) -> Result<usize>;
    fn get_uint64(&self, option: NngOption) -> Result<u64>;
    fn get_string(&self, option: NngOption) -> Result<NngString>;
}

/// Types which support setting NNG options.
/// Also see [`GetOpts`](trait.GetOpts.html).
pub trait SetOpts {
    fn set_bool(&mut self, option: NngOption, value: bool) -> Result<&mut Self>;
    fn set_int(&mut self, option: NngOption, value: i32) -> Result<&mut Self>;
    fn set_ms(&mut self, option: NngOption, value: nng_duration) -> Result<&mut Self>;
    fn set_size(&mut self, option: NngOption, value: usize) -> Result<&mut Self>;
    fn set_uint64(&mut self, option: NngOption, value: u64) -> Result<&mut Self>;
    fn set_string(&mut self, option: NngOption, value: &str) -> Result<&mut Self>;

    fn set_duration(&mut self, option: NngOption, value: time::Duration) -> Result<&mut Self> {
        let ms = value.as_millis() as nng_duration;
        self.set_ms(option, ms)
    }
}

/// Wraps NNG option names for [GetOpts](trait.GetOpts.html) and [SetOpts](trait.SetOpts.html).
/// See [nng_options](https://nanomsg.github.io/nng/man/v1.1.0/nng_options.5).
#[derive(Debug, PartialEq)]
pub struct NngOption(&'static [u8]);

impl NngOption {
    /// Return option name suitable for passing to C functions.
    pub fn as_cptr(&self) -> *const c_char {
        self.0.as_ptr() as *const c_char
    }

    pub const SOCKNAME: NngOption = NngOption(NNG_OPT_SOCKNAME);
    pub const RAW: NngOption = NngOption(NNG_OPT_RAW);
    pub const PROTO: NngOption = NngOption(NNG_OPT_PROTO);
    pub const PROTONAME: NngOption = NngOption(NNG_OPT_PROTONAME);
    pub const PEER: NngOption = NngOption(NNG_OPT_PEER);
    pub const PEERNAME: NngOption = NngOption(NNG_OPT_PEERNAME);
    pub const RECVBUF: NngOption = NngOption(NNG_OPT_RECVBUF);
    pub const SENDBUF: NngOption = NngOption(NNG_OPT_SENDBUF);
    pub const RECVFD: NngOption = NngOption(NNG_OPT_RECVFD);
    pub const SENDFD: NngOption = NngOption(NNG_OPT_SENDFD);
    pub const RECVTIMEO: NngOption = NngOption(NNG_OPT_RECVTIMEO);
    pub const SENDTIMEO: NngOption = NngOption(NNG_OPT_SENDTIMEO);
    pub const LOCADDR: NngOption = NngOption(NNG_OPT_LOCADDR);
    pub const REMADDR: NngOption = NngOption(NNG_OPT_REMADDR);
    pub const URL: NngOption = NngOption(NNG_OPT_URL);
    pub const MAXTTL: NngOption = NngOption(NNG_OPT_MAXTTL);
    pub const RECVMAXSZ: NngOption = NngOption(NNG_OPT_RECVMAXSZ);
    pub const RECONNMINT: NngOption = NngOption(NNG_OPT_RECONNMINT);
    pub const RECONNMAXT: NngOption = NngOption(NNG_OPT_RECONNMAXT);
    pub const TLS_CONFIG: NngOption = NngOption(NNG_OPT_TLS_CONFIG);
    pub const TLS_AUTH_MODE: NngOption = NngOption(NNG_OPT_TLS_AUTH_MODE);
    pub const TLS_CERT_KEY_FILE: NngOption = NngOption(NNG_OPT_TLS_CERT_KEY_FILE);
    pub const TLS_CA_FILE: NngOption = NngOption(NNG_OPT_TLS_CA_FILE);
    pub const TLS_SERVER_NAME: NngOption = NngOption(NNG_OPT_TLS_SERVER_NAME);
    pub const TLS_VERIFIED: NngOption = NngOption(NNG_OPT_TLS_VERIFIED);
    pub const TCP_NODELAY: NngOption = NngOption(NNG_OPT_TCP_NODELAY);
    pub const TCP_KEEPALIVE: NngOption = NngOption(NNG_OPT_TCP_KEEPALIVE);
    pub const TCP_BOUND_PORT: NngOption = NngOption(NNG_OPT_TCP_BOUND_PORT);
    pub const IPC_SECURITY_DESCRIPTOR: NngOption = NngOption(NNG_OPT_IPC_SECURITY_DESCRIPTOR);
    pub const IPC_PERMISSIONS: NngOption = NngOption(NNG_OPT_IPC_PERMISSIONS);
    pub const IPC_PEER_UID: NngOption = NngOption(NNG_OPT_IPC_PEER_UID);
    pub const IPC_PEER_GID: NngOption = NngOption(NNG_OPT_IPC_PEER_GID);
    pub const IPC_PEER_PID: NngOption = NngOption(NNG_OPT_IPC_PEER_PID);
    pub const IPC_PEER_ZONEID: NngOption = NngOption(NNG_OPT_IPC_PEER_ZONEID);
    pub const WS_REQUEST_HEADERS: NngOption = NngOption(NNG_OPT_WS_REQUEST_HEADERS);
    pub const WS_RESPONSE_HEADERS: NngOption = NngOption(NNG_OPT_WS_RESPONSE_HEADERS);
    pub const WS_RESPONSE_HEADER: NngOption = NngOption(NNG_OPT_WS_RESPONSE_HEADER);
    pub const WS_REQUEST_HEADER: NngOption = NngOption(NNG_OPT_WS_REQUEST_HEADER);
    pub const WS_REQUEST_URI: NngOption = NngOption(NNG_OPT_WS_REQUEST_URI);
    pub const WS_SENDMAXFRAME: NngOption = NngOption(NNG_OPT_WS_SENDMAXFRAME);
    pub const WS_RECVMAXFRAME: NngOption = NngOption(NNG_OPT_WS_RECVMAXFRAME);
    pub const WS_PROTOCOL: NngOption = NngOption(NNG_OPT_WS_PROTOCOL);
    pub const PAIR1_POLY: NngOption = NngOption(NNG_OPT_PAIR1_POLY);
    pub const SUB_SUBSCRIBE: NngOption = NngOption(NNG_OPT_SUB_SUBSCRIBE);
    pub const SUB_UNSUBSCRIBE: NngOption = NngOption(NNG_OPT_SUB_UNSUBSCRIBE);
    pub const REQ_RESENDTIME: NngOption = NngOption(NNG_OPT_REQ_RESENDTIME);
    pub const SURVEYOR_SURVEYTIME: NngOption = NngOption(NNG_OPT_SURVEYOR_SURVEYTIME);
    pub const WSS_REQUEST_HEADERS: NngOption = NngOption(NNG_OPT_WSS_REQUEST_HEADERS);
    pub const WSS_RESPONSE_HEADERS: NngOption = NngOption(NNG_OPT_WSS_RESPONSE_HEADERS);
    pub const ZT_HOME: NngOption = NngOption(NNG_OPT_ZT_HOME);
    pub const ZT_NWID: NngOption = NngOption(NNG_OPT_ZT_NWID);
    pub const ZT_NODE: NngOption = NngOption(NNG_OPT_ZT_NODE);
    pub const ZT_NETWORK_STATUS: NngOption = NngOption(NNG_OPT_ZT_NETWORK_STATUS);
    pub const ZT_NETWORK_NAME: NngOption = NngOption(NNG_OPT_ZT_NETWORK_NAME);
    pub const ZT_PING_TIME: NngOption = NngOption(NNG_OPT_ZT_PING_TIME);
    pub const ZT_PING_TRIES: NngOption = NngOption(NNG_OPT_ZT_PING_TRIES);
    pub const ZT_CONN_TIME: NngOption = NngOption(NNG_OPT_ZT_CONN_TIME);
    pub const ZT_CONN_TRIES: NngOption = NngOption(NNG_OPT_ZT_CONN_TRIES);
    pub const ZT_MTU: NngOption = NngOption(NNG_OPT_ZT_MTU);
    pub const ZT_ORBIT: NngOption = NngOption(NNG_OPT_ZT_ORBIT);
    pub const ZT_DEORBIT: NngOption = NngOption(NNG_OPT_ZT_DEORBIT);
    pub const ZT_ADD_LOCAL_ADDR: NngOption = NngOption(NNG_OPT_ZT_ADD_LOCAL_ADDR);
    pub const ZT_CLEAR_LOCAL_ADDRS: NngOption = NngOption(NNG_OPT_ZT_CLEAR_LOCAL_ADDRS);
}
