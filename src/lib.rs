pub mod errno;
pub mod http;
pub mod server;
pub mod syscalls;

pub const AF_INET: u16 = 2;
pub const S_TYPE: u16 = 1;
pub const PROTOCOL: u16 = 0;
pub const PORT: u16 = 8080;
pub const INADDR_ANY: u32 = 0x00000000;
pub const INADDR_LOOPBACK: u32 = 0x7f000001;
