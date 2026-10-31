use crate::errno::{Errno, check};
use std::arch::asm;

pub const SYS_READ: i64 = 0;
pub const SYS_WRITE: i64 = 1;
pub const SYS_CLOSE: i64 = 3;
pub const SYS_SOCKET: i64 = 41;
pub const SYS_CONNECT: i64 = 42;
pub const SYS_ACCEPT: i64 = 43;
pub const SYS_SENDTO: i64 = 44;
pub const SYS_BIND: i64 = 49;
pub const SYS_LISTEN: i64 = 50;
pub const SYS_GETSOCKNAME: i64 = 51;
pub const SYS_SETSOCKOPT: i64 = 54;

pub const SOL_SOCKET: i64 = 1;
pub const SO_REUSEADDR: i64 = 2;
pub const SO_RCVTIMEO: i64 = 20;
pub const MSG_NOSIGNAL: i64 = 0x4000;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct SockaddrIn {
    pub sin_family: u16,
    pub sin_port: u16,
    pub sin_addr: u32,
    pub padding: [u8; 8],
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Timeval {
    pub tv_sec: i64,
    pub tv_usec: i64,
}

pub const fn sockaddr_in(family: u16, addr: u32, port: u16) -> SockaddrIn {
    SockaddrIn {
        sin_family: family,
        sin_port: port.to_be(),
        sin_addr: addr.to_be(),
        padding: [0; 8],
    }
}

unsafe fn syscall1(nr: i64, a: i64) -> i64 {
    let ret: i64;
    unsafe {
        asm!(
            "syscall",
            inlateout("rax") nr => ret,
            in("rdi") a,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack),
        );
    }
    ret
}

unsafe fn syscall3(nr: i64, a: i64, b: i64, c: i64) -> i64 {
    let ret: i64;
    unsafe {
        asm!(
            "syscall",
            inlateout("rax") nr => ret,
            in("rdi") a,
            in("rsi") b,
            in("rdx") c,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack),
        );
    }
    ret
}

unsafe fn syscall6(nr: i64, a: i64, b: i64, c: i64, d: i64, e: i64, f: i64) -> i64 {
    let ret: i64;
    unsafe {
        asm!(
            "syscall",
            inlateout("rax") nr => ret,
            in("rdi") a,
            in("rsi") b,
            in("rdx") c,
            in("r10") d,
            in("r8") e,
            in("r9") f,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack),
        );
    }
    ret
}

pub fn socket(domain: u16, s_type: u16, protocol: u16) -> Result<i64, Errno> {
    check(unsafe { syscall3(SYS_SOCKET, domain as i64, s_type as i64, protocol as i64) })
}

pub fn bind(fd: i64, addr: &SockaddrIn) -> Result<(), Errno> {
    let len = size_of::<SockaddrIn>() as i64;
    check(unsafe { syscall3(SYS_BIND, fd, addr as *const SockaddrIn as i64, len) })?;
    Ok(())
}

pub fn listen(fd: i64, backlog: i64) -> Result<(), Errno> {
    check(unsafe { syscall3(SYS_LISTEN, fd, backlog, 0) })?;
    Ok(())
}

pub fn accept(fd: i64, peer: &mut SockaddrIn) -> Result<i64, Errno> {
    // socklen_t is u32
    let mut len: u32 = size_of::<SockaddrIn>() as u32;
    let peer_ptr = peer as *mut SockaddrIn as i64;
    let len_ptr = &mut len as *mut u32 as i64;
    check(unsafe { syscall3(SYS_ACCEPT, fd, peer_ptr, len_ptr) })
}

pub fn connect(fd: i64, addr: &SockaddrIn) -> Result<(), Errno> {
    let len = size_of::<SockaddrIn>() as i64;
    check(unsafe { syscall3(SYS_CONNECT, fd, addr as *const SockaddrIn as i64, len) })?;
    Ok(())
}

pub fn getsockname(fd: i64, addr: &mut SockaddrIn) -> Result<(), Errno> {
    let mut len: u32 = size_of::<SockaddrIn>() as u32;
    let addr_ptr = addr as *mut SockaddrIn as i64;
    let len_ptr = &mut len as *mut u32 as i64;
    check(unsafe { syscall3(SYS_GETSOCKNAME, fd, addr_ptr, len_ptr) })?;
    Ok(())
}

pub fn setsockopt<T>(fd: i64, level: i64, name: i64, value: &T) -> Result<(), Errno> {
    let ptr = value as *const T as i64;
    let len = size_of::<T>() as i64;
    check(unsafe { syscall6(SYS_SETSOCKOPT, fd, level, name, ptr, len, 0) })?;
    Ok(())
}

pub fn read(fd: i64, buf: &mut [u8]) -> Result<usize, Errno> {
    let n = check(unsafe { syscall3(SYS_READ, fd, buf.as_mut_ptr() as i64, buf.len() as i64) })?;
    Ok(n as usize)
}

pub fn write(fd: i64, buf: &[u8]) -> Result<usize, Errno> {
    let n = check(unsafe { syscall3(SYS_WRITE, fd, buf.as_ptr() as i64, buf.len() as i64) })?;
    Ok(n as usize)
}

// MSG_NOSIGNAL: EPIPE instead of SIGPIPE
pub fn send(fd: i64, buf: &[u8]) -> Result<usize, Errno> {
    let ptr = buf.as_ptr() as i64;
    let n = check(unsafe { syscall6(SYS_SENDTO, fd, ptr, buf.len() as i64, MSG_NOSIGNAL, 0, 0) })?;
    Ok(n as usize)
}

pub fn send_all(fd: i64, buf: &[u8]) -> Result<(), Errno> {
    let mut sent = 0;
    while sent < buf.len() {
        match send(fd, &buf[sent..]) {
            Ok(n) => sent += n,
            Err(Errno::EINTR) => continue,
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

pub fn close(fd: i64) -> Result<(), Errno> {
    check(unsafe { syscall1(SYS_CLOSE, fd) })?;
    Ok(())
}
