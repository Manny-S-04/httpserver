use std::arch::asm;
use crate::BUF_COUNT;

#[repr(C)]
#[derive(Debug)]
pub struct SockaddrIn {
   pub sin_family: u16,
   pub sin_port:   u16,
   pub sin_addr:   u32,
   pub padding:    [u8; 8],
}

impl Default for SockaddrIn {
    fn default() -> Self {
        Self {
            sin_family: Default::default(),
            sin_port: Default::default(),
            sin_addr: Default::default(),
            padding: Default::default() 
        }
    }
}


pub fn socket(domain: i64, s_type: i64, protocol: i64) -> i64 {
    unsafe {
        let ret: i64;
        asm!(
            "syscall",
            inlateout("rax") 41i64 => ret,
            in("rdi") domain,
            in("rsi") s_type,
            in("rdx") protocol,
            lateout("rcx") _,
            lateout("r11") _,
        );
        ret
    }
}

pub fn bind(fd: i64, addr: &SockaddrIn, len: i64) -> i64 {
    unsafe {
        let ret: i64;
        asm!(
            "syscall",
            inlateout("rax") 49i64 => ret,
            in("rdi") fd,
            in("rsi") addr,
            in("rdx") len,
            lateout("rcx") _,
            lateout("r11") _,
        );
        ret
    }
}

pub fn listen(fd: i64, backlog: i64) -> i64 {
    unsafe {
        let ret: i64;
        asm!(
            "syscall",
            inlateout("rax") 50i64 => ret,
            in("rdi") fd,
            in("rsi") backlog,
            lateout("rcx") _,
            lateout("r11") _,
        );
        ret
    }
}

pub fn accept(fd: i64, upeer_sockaddr: &mut SockaddrIn, upeer_addrlen: &mut i64) -> i64 {
    unsafe {
        let ret: i64;
        asm!(
            "syscall",
            inlateout("rax") 43i64 => ret,
            in("rdi") fd,
            in("rsi") upeer_sockaddr,
            in("rdx") upeer_addrlen,
            lateout("rcx") _,
            lateout("r11") _,
        );
        ret
    }
}

pub fn read(fd: i64, buf: &mut [u8; BUF_COUNT], count: usize) -> isize {
    unsafe {
        let ret: isize;
        asm!(
            "syscall",
            inlateout("rax") 0i64 => ret,
            in("rdi") fd,
            in("rsi") buf,
            in("rdx") count,
            lateout("rcx") _,
            lateout("r11") _,
        );
        ret
    }
}

pub fn write(fd: i64, buf: &[u8], count: isize) -> i64 {
    unsafe {
        let ret: i64;
        asm!(
            "syscall",
            inlateout("rax") 1i64 => ret,
            in("rdi") fd,
            in("rsi") buf.as_ptr(),
            in("rdx") count,
            lateout("rcx") _,
            lateout("r11") _,
        );
        ret
    }
}

pub fn connect(fd: i64, addr: &mut SockaddrIn, addrlen: i64) -> i64 {
    unsafe {
        let ret: i64;
        asm!(
            "syscall",
            inlateout("rax") 42i64 => ret,
            in("rdi") fd,
            in("rsi") addr,
            in("rdx") addrlen,
            lateout("rcx") _,
            lateout("r11") _,
        );
        ret
    }
}

pub fn close(fd: i64) -> i64 {
    unsafe {
        let ret: i64;
        asm!(
            "syscall",
            inlateout("rax") 3i64 => ret,
            in("rdi") fd,
            lateout("rcx") _,
            lateout("r11") _,
        );
        ret
    }
}
