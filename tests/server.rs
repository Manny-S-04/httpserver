use httpserver::errno::Errno;
use httpserver::http::{Request, Response, status_page, text};
use httpserver::server::{listen_on, local_port, serve};
use httpserver::syscalls::close;
use httpserver::*;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

fn route(req: &Request) -> Response {
    match req.path.as_str() {
        "/" => text(200, "hello"),
        "/echo" => Response {
            status: 200,
            content_type: "application/octet-stream",
            body: req.body.clone(),
        },
        _ => status_page(404),
    }
}

fn start(connections: usize) -> (u16, thread::JoinHandle<i64>) {
    let fd = listen_on(INADDR_LOOPBACK, 0).unwrap();
    let port = local_port(fd).unwrap();
    let handle = thread::spawn(move || {
        serve(fd, route, connections).unwrap();
        fd
    });
    (port, handle)
}

fn request(port: u16, parts: &[&[u8]]) -> String {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    // fail instead of hang if a fd is never closed
    s.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    for p in parts {
        s.write_all(p).unwrap();
        s.flush().unwrap();
        thread::sleep(Duration::from_millis(5));
    }
    let mut out = String::new();
    s.read_to_string(&mut out).unwrap();
    out
}

fn open_fds() -> usize {
    std::fs::read_dir("/proc/self/fd").unwrap().count()
}

#[test]
fn split_request() {
    let (port, handle) = start(1);
    let out = request(port, &[b"GE", b"T / HTTP/1.1\r\nHo", b"st: x\r\n", b"\r\n"]);
    assert!(out.starts_with("HTTP/1.1 200 OK\r\n"), "{out}");
    assert!(out.contains("Content-Length: 5\r\n"));
    assert!(out.ends_with("\r\n\r\nhello"));
    close(handle.join().unwrap()).unwrap();
}

#[test]
fn body_across_reads() {
    let (port, handle) = start(1);
    let out = request(
        port,
        &[
            b"POST /echo HTTP/1.1\r\nHost: x\r\nContent-Length: 11\r\n\r\nhello",
            b" world",
        ],
    );
    assert!(out.ends_with("\r\n\r\nhello world"), "{out}");
    close(handle.join().unwrap()).unwrap();
}

#[test]
fn errors() {
    let (port, handle) = start(4);
    assert!(request(port, &[b"nonsense\r\n\r\n"]).starts_with("HTTP/1.1 400 "));
    assert!(request(port, &[b"GET / HTTP/1.1\r\n\r\n"]).starts_with("HTTP/1.1 400 "));
    assert!(request(port, &[b"GET / HTTP/3.0\r\nHost: x\r\n\r\n"]).starts_with("HTTP/1.1 505 "));
    assert!(
        request(port, &[b"GET /nope HTTP/1.1\r\nHost: x\r\n\r\n"]).starts_with("HTTP/1.1 404 ")
    );
    close(handle.join().unwrap()).unwrap();
}

#[test]
fn head_has_no_body() {
    let (port, handle) = start(1);
    let out = request(port, &[b"HEAD / HTTP/1.1\r\nHost: x\r\n\r\n"]);
    assert!(out.contains("Content-Length: 5\r\n"));
    assert!(out.ends_with("\r\n\r\n"), "{out}");
    close(handle.join().unwrap()).unwrap();
}

#[test]
fn client_hangs_up() {
    let (port, handle) = start(2);
    drop(TcpStream::connect(("127.0.0.1", port)).unwrap());
    assert!(request(port, &[b"GET / HTTP/1.0\r\n\r\n"]).starts_with("HTTP/1.1 200 "));
    close(handle.join().unwrap()).unwrap();
}

#[test]
fn many_connections_no_fd_leak() {
    let before = open_fds();
    let (port, handle) = start(200);
    for _ in 0..200 {
        assert!(request(port, &[b"GET / HTTP/1.1\r\nHost: x\r\n\r\n"]).ends_with("hello"));
    }
    let fd = handle.join().unwrap();
    close(fd).unwrap();
    // other tests run in parallel
    assert!(
        open_fds() <= before + 8,
        "before {before} after {}",
        open_fds()
    );
    assert_eq!(close(fd), Err(Errno::EBADF));
}

#[test]
fn address_in_use() {
    let fd = listen_on(INADDR_LOOPBACK, 0).unwrap();
    let port = local_port(fd).unwrap();
    assert_eq!(listen_on(INADDR_LOOPBACK, port), Err(Errno::EADDRINUSE));
    close(fd).unwrap();
}
