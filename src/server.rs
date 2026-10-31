use crate::errno::Errno;
use crate::http::{self, Parse, Request, Response};
use crate::syscalls::*;
use crate::{AF_INET, PROTOCOL, S_TYPE};

pub const BACKLOG: i64 = 128;
pub const READ_TIMEOUT_SECS: i64 = 5;
pub const READ_CHUNK: usize = 4096;

pub type Handler = fn(&Request) -> Response;

pub fn listen_on(addr: u32, port: u16) -> Result<i64, Errno> {
    let fd = socket(AF_INET, S_TYPE, PROTOCOL)?;
    match setup_listener(fd, addr, port) {
        Ok(()) => Ok(fd),
        Err(e) => {
            let _ = close(fd);
            Err(e)
        }
    }
}

fn setup_listener(fd: i64, addr: u32, port: u16) -> Result<(), Errno> {
    let one: i32 = 1;
    setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &one)?;
    bind(fd, &sockaddr_in(AF_INET, addr, port))?;
    listen(fd, BACKLOG)
}

pub fn local_port(fd: i64) -> Result<u16, Errno> {
    let mut addr = SockaddrIn::default();
    getsockname(fd, &mut addr)?;
    Ok(u16::from_be(addr.sin_port))
}

pub fn serve(fd: i64, handler: Handler, max_connections: usize) -> Result<(), Errno> {
    let mut served = 0;
    while served < max_connections {
        let mut peer = SockaddrIn::default();
        let conn = match accept(fd, &mut peer) {
            Ok(conn) => conn,
            Err(Errno::EINTR) | Err(Errno::ECONNABORTED) => continue,
            Err(e) => return Err(e),
        };

        let res = handle_connection(conn, handler);
        let _ = close(conn);
        if let Err(e) = res {
            let _ = write(2, format!("connection: {e:?}\n").as_bytes());
        }
        served += 1;
    }
    Ok(())
}

pub fn handle_connection(fd: i64, handler: Handler) -> Result<(), Errno> {
    let timeout = Timeval {
        tv_sec: READ_TIMEOUT_SECS,
        tv_usec: 0,
    };
    setsockopt(fd, SOL_SOCKET, SO_RCVTIMEO, &timeout)?;

    let mut buf: Vec<u8> = Vec::new();
    let mut chunk = [0u8; READ_CHUNK];
    loop {
        match http::parse_request(&buf) {
            Parse::Done(req, _) => {
                let res = handler(&req);
                log_request(&req, &res);
                return send_all(fd, &http::response_bytes(&res, req.method == "HEAD"));
            }
            Parse::Error(e) => {
                let res = http::status_page(http::error_status(e));
                return send_all(fd, &http::response_bytes(&res, false));
            }
            Parse::Incomplete => {}
        }

        match read(fd, &mut chunk) {
            Ok(0) => return Ok(()),
            Ok(n) => buf.extend_from_slice(&chunk[..n]),
            Err(Errno::EINTR) => continue,
            Err(Errno::EAGAIN) => {
                if buf.is_empty() {
                    return Ok(());
                }
                let res = http::status_page(408);
                return send_all(fd, &http::response_bytes(&res, false));
            }
            Err(e) => return Err(e),
        }
    }
}

fn log_request(req: &Request, res: &Response) {
    let line = format!("{} {} {}\n", req.method, req.path, res.status);
    let _ = write(1, line.as_bytes());
}
