use http::*;
use syscalls::*;

fn main() {
    let client_fd = client(AF_INET, S_TYPE, PROTOCOL, PORT, INADDR_ANY);
    let mut input = String::new();
    loop {
        input.clear();
        std::io::stdin().read_line(&mut input).unwrap();
        let line = input.trim_end();
        if line.is_empty() {
            continue;
        }

        let bytes = line.as_bytes();
        if bytes.len() > BUF_COUNT {
            eprintln!("line too long, dropping");
            continue;
        }

        let mut buf = [0u8; BUF_COUNT];
        buf[..bytes.len()].copy_from_slice(bytes);

        let mut sent = 0usize;
        while sent < bytes.len() {
            let remaining = &buf[sent..bytes.len()];
            let res = write(client_fd, remaining, remaining.len() as isize);

            if res < 0 {
                eprintln!("write failed: {res}");
                break;
            }

            sent += res as usize;
        }

        if line == "close" {
            close(client_fd);
            break;
        }
    }
}

fn client(sa_family: u16, sa_type: u16, protocol: u16, port: u16, addr: u32) -> i64 {
    let fd: i64 = socket(sa_family as i64, sa_type as i64, protocol as i64);
    let mut sockaddr_in =
    SockaddrIn {
        sin_family: sa_family,
        sin_port: port.to_be(), // to big endian
        sin_addr: addr,
        padding: [0; 8]
    };
    let len = size_of::<SockaddrIn>() as i64;
    let res = connect(fd, &mut sockaddr_in, len);
    assert_eq!(res, 0);
    fd
}
