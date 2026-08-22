mod syscalls;
use syscalls::*;

const AF_INET: u16 = 2;
const S_TYPE: u16 = 1;
const PROTOCOL: u16 = 0;
const PORT: u16 = 8080;
const INADDR_ANY: u32 = 0x00000000;

fn main() {
    let socket_fd = server(AF_INET, S_TYPE, PROTOCOL, PORT, INADDR_ANY);
    let mut buf = [0 as u8; BUF_COUNT];
    let res = read(socket_fd, &mut buf, BUF_COUNT); // hangs app
    let _ = write(1, &buf, res);
}

fn client() {

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


/*
43	sys_accept	int fd	struct sockaddr *upeer_sockaddr	int *upeer_addrlen
*/
/*
Server
socket -> bind -> listen -> accept -> read/write
Client
socket -> connect -> read/write
*/
