//! NNG options.  See [nng_options](https://nanomsg.github.io/nng/man/v1.1.0/nng_options.5).

use super::*;

pub struct NngString {
    pointer: *mut ::std::os::raw::c_char,
}

impl NngString {
    pub fn new(pointer: *mut ::std::os::raw::c_char) -> NngString {
        NngString { pointer }
    }
    pub fn to_str(&self) -> Result<&str, std::str::Utf8Error> {
        unsafe { std::ffi::CStr::from_ptr(self.pointer).to_str() }
    }
}

impl Drop for NngString {
    fn drop(&mut self) {
        unsafe {
            nng_strfree(self.pointer);
        }
    }
}

/// Trait for types which support getting NNG options.
pub trait GetOpts {
    fn getopt_bool(&self, option: NngOption) -> NngResult<bool>;
    fn getopt_int(&self, option: NngOption) -> NngResult<i32>;
    fn getopt_ms(&self, option: NngOption) -> NngResult<nng_duration>;
    fn getopt_size(&self, option: NngOption) -> NngResult<usize>;
    fn getopt_uint64(&self, option: NngOption) -> NngResult<u64>;
    fn getopt_string(&self, option: NngOption) -> NngResult<NngString>;
}

/// Trait for types which support setting NNG options.
pub trait SetOpts {
    fn setopt_bool(&mut self, option: NngOption, value: bool) -> NngReturn;
    fn setopt_int(&mut self, option: NngOption, value: i32) -> NngReturn;
    fn setopt_ms(&mut self, option: NngOption, value: nng_duration) -> NngReturn;
    fn setopt_size(&mut self, option: NngOption, value: usize) -> NngReturn;
    fn setopt_uint64(&mut self, option: NngOption, value: u64) -> NngReturn;
    fn setopt_string(&mut self, option: NngOption, value: &str) -> NngReturn;
}

pub struct NngOption(&'static [u8]);

impl NngOption {
    /// Return option name as `const char*` suitable for passing to C functions.
    pub fn as_cptr(&self) -> *const ::std::os::raw::c_char {
        self.0.as_ptr() as *const ::std::os::raw::c_char
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
    pub const PAIR1_POLY: NngOption = NngOption(NNG_OPT_PAIR1_POLY);
    pub const SUB_SUBSCRIBE: NngOption = NngOption(NNG_OPT_SUB_SUBSCRIBE);
    pub const SUB_UNSUBSCRIBE: NngOption = NngOption(NNG_OPT_SUB_UNSUBSCRIBE);
    pub const REQ_RESENDTIME: NngOption = NngOption(NNG_OPT_REQ_RESENDTIME);
    pub const SURVEYOR_SURVEYTIME: NngOption = NngOption(NNG_OPT_SURVEYOR_SURVEYTIME);
    pub const IPC_SECURITY_DESCRIPTOR: NngOption = NngOption(NNG_OPT_IPC_SECURITY_DESCRIPTOR);
    pub const IPC_PERMISSIONS: NngOption = NngOption(NNG_OPT_IPC_PERMISSIONS);
    pub const IPC_PEER_UID: NngOption = NngOption(NNG_OPT_IPC_PEER_UID);
    pub const IPC_PEER_GID: NngOption = NngOption(NNG_OPT_IPC_PEER_GID);
    pub const IPC_PEER_PID: NngOption = NngOption(NNG_OPT_IPC_PEER_PID);
    pub const IPC_PEER_ZONEID: NngOption = NngOption(NNG_OPT_IPC_PEER_ZONEID);
    pub const WS_REQUEST_HEADERS: NngOption = NngOption(NNG_OPT_WS_REQUEST_HEADERS);
    pub const WS_RESPONSE_HEADERS: NngOption = NngOption(NNG_OPT_WS_RESPONSE_HEADERS);
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
