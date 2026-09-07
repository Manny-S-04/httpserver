use http::*;
use http::syscalls::*;

fn main() {
    let server_fd = server(AF_INET, S_TYPE, PROTOCOL, PORT, INADDR_ANY);
    loop {
        let mut buf = [0 as u8; BUF_COUNT];
        let res = read(server_fd, &mut buf, BUF_COUNT); // hangs app
        if res <= 0 {
            break; // EOF or error — client's gone
        }
        let n = res as usize;
        let _ = write(1, &buf[..n], res);
        let _ = write(1, b"\n", res);

        if &buf[..n] == b"close" {
            close(server_fd);
            break;
        }
    }
}

fn server(sa_family: u16, sa_type: u16, protocol: u16, port: u16, addr: u32) -> i64 {
    let fd: i64 = socket(sa_family as i64, sa_type as i64, protocol as i64);
    let sockaddr_in =
    SockaddrIn {
        sin_family: sa_family,
        sin_port: port.to_be(), // to big endian
        sin_addr: addr,
        padding: [0; 8]
    };

    let b = bind(fd, &sockaddr_in, size_of::<SockaddrIn>() as i64);
    assert_eq!(b, 0);
    let l = listen(fd, 10);
    assert_eq!(l, 0);
    let mut upeer = SockaddrIn::default();
    let mut upeer_len = size_of::<SockaddrIn>() as i64;
    accept(fd, &mut upeer, &mut upeer_len)
}
