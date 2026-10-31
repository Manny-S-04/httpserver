use httpserver::syscalls::*;
use httpserver::*;

fn main() {
    let path = std::env::args().nth(1).unwrap_or("/".to_string());
    let fd = match client(INADDR_LOOPBACK, PORT) {
        Ok(fd) => fd,
        Err(e) => {
            eprintln!("connect to {PORT}: {e:?}");
            std::process::exit(1);
        }
    };

    let req = format!("GET {path} HTTP/1.1\r\nHost: localhost:{PORT}\r\n\r\n");
    if let Err(e) = send_all(fd, req.as_bytes()) {
        eprintln!("send: {e:?}");
    }

    let mut buf = [0u8; 4096];
    loop {
        match read(fd, &mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let _ = write(1, &buf[..n]);
            }
            Err(e) => {
                eprintln!("read: {e:?}");
                break;
            }
        }
    }
    let _ = close(fd);
}

fn client(addr: u32, port: u16) -> Result<i64, errno::Errno> {
    let fd = socket(AF_INET, S_TYPE, PROTOCOL)?;
    if let Err(e) = connect(fd, &sockaddr_in(AF_INET, addr, port)) {
        let _ = close(fd);
        return Err(e);
    }
    Ok(fd)
}
