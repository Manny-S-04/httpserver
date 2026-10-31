# httpserver

HTTP/1.1 server in Rust with no crates, no `std::net` and no libc. Every socket call is a raw Linux syscall written with inline asm. Wanted to see what's actually under `TcpListener::bind`.

WIP: it accepts connections one at a time, parses the request line and headers (even when they arrive over several reads), and sends back a response with a Content-Length. One request per connection for now.

## Run

Linux x86-64 only.

```sh
cargo run --bin server        # listens on 0.0.0.0:8080
curl -i localhost:8080/
cargo run --bin client /nope  # sends a GET and prints the response
cargo test
```

## Layout

- `src/syscalls.rs` - `syscall1/3/6` in asm, then socket, bind, listen, accept, connect, read, write, send, close, getsockname, setsockopt on top
- `src/errno.rs` - `Errno` enum, `check` turns a negative return into `Err`
- `src/http.rs` - request parsing and response bytes
- `src/server.rs` - listener setup, accept loop, one connection
- `tests/server.rs` - real sockets: split requests, bodies, errors, 200 connections with no fd leak

## Syscalls

| syscall | nr |
|---|---|
| read / write | 0 / 1 |
| close | 3 |
| socket | 41 |
| connect | 42 |
| accept | 43 |
| sendto | 44 |
| bind | 49 |
| listen | 50 |
| getsockname | 51 |
| setsockopt | 54 |

Things I got caught on:

- `SockaddrIn` needs `#[repr(C)]` so it matches the kernel's 16 byte `sockaddr_in`
- the port and the address have to be big endian, `port.to_be()` is basically `htons`
- errors come back as negative errno since there's no libc, e.g. -98 for EADDRINUSE. Anything from -4095 to -1 is an error
- `accept` gives you a new fd for the connection, the listening fd stays separate, and both need closing
- `socklen_t` is 32 bits. I passed an `i64` and it only worked because of little endian
- writing to a socket the client already closed sends SIGPIPE and kills the process. `sendto` with `MSG_NOSIGNAL` gives EPIPE instead
- a client that connects and never sends anything blocks everything, so connections get a 5 second `SO_RCVTIMEO` and a 408

## Parsing

`parse_request` gets called with the whole buffer after every read and returns `Incomplete`, `Done(request, bytes_used)` or `Error`. Nothing is kept between calls, so a request split anywhere parses the same as one that arrives in one go. There's a test that feeds every prefix of a request and checks they're all `Incomplete`.

It follows RFC 9112 where it's cheap: HTTP/1.1 needs a Host header, no space before the colon in a header, no folded header lines, two different Content-Lengths is a 400, Transfer-Encoding is a 501 for now. Heads over 8KB get a 431, bodies over 1MB a 413.

## Todo

- [x] syscall layer
- [x] client and server sending bytes
- [x] errno enum instead of `assert_eq!`
- [x] accept loop for more than one connection, close each fd
- [x] parse the request line and headers (requests can span multiple reads)
- [x] responses with content-length
- [ ] static files
- [ ] chunked bodies
- [ ] keep-alive, then threads, then epoll
- [ ] benchmark against std::net with wrk
