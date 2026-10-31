#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Errno {
    EINTR,
    EBADF,
    EAGAIN,
    ENOMEM,
    EACCES,
    EFAULT,
    EINVAL,
    ENFILE,
    EMFILE,
    EPIPE,
    ENOTSOCK,
    EADDRINUSE,
    EADDRNOTAVAIL,
    ECONNABORTED,
    ECONNRESET,
    ENOTCONN,
    ETIMEDOUT,
    ECONNREFUSED,
    Other(i64),
}

pub const fn from_code(code: i64) -> Errno {
    match code {
        4 => Errno::EINTR,
        9 => Errno::EBADF,
        11 => Errno::EAGAIN,
        12 => Errno::ENOMEM,
        13 => Errno::EACCES,
        14 => Errno::EFAULT,
        22 => Errno::EINVAL,
        23 => Errno::ENFILE,
        24 => Errno::EMFILE,
        32 => Errno::EPIPE,
        88 => Errno::ENOTSOCK,
        98 => Errno::EADDRINUSE,
        99 => Errno::EADDRNOTAVAIL,
        103 => Errno::ECONNABORTED,
        104 => Errno::ECONNRESET,
        107 => Errno::ENOTCONN,
        110 => Errno::ETIMEDOUT,
        111 => Errno::ECONNREFUSED,
        n => Errno::Other(n),
    }
}

pub const fn code(e: Errno) -> i64 {
    match e {
        Errno::EINTR => 4,
        Errno::EBADF => 9,
        Errno::EAGAIN => 11,
        Errno::ENOMEM => 12,
        Errno::EACCES => 13,
        Errno::EFAULT => 14,
        Errno::EINVAL => 22,
        Errno::ENFILE => 23,
        Errno::EMFILE => 24,
        Errno::EPIPE => 32,
        Errno::ENOTSOCK => 88,
        Errno::EADDRINUSE => 98,
        Errno::EADDRNOTAVAIL => 99,
        Errno::ECONNABORTED => 103,
        Errno::ECONNRESET => 104,
        Errno::ENOTCONN => 107,
        Errno::ETIMEDOUT => 110,
        Errno::ECONNREFUSED => 111,
        Errno::Other(n) => n,
    }
}

// -4095..=-1 is -errno
pub const fn check(ret: i64) -> Result<i64, Errno> {
    if ret < 0 && ret > -4096 {
        Err(from_code(-ret))
    } else {
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_is_error() {
        assert_eq!(check(-98), Err(Errno::EADDRINUSE));
        assert_eq!(check(-4), Err(Errno::EINTR));
        assert_eq!(check(-200), Err(Errno::Other(200)));
    }

    #[test]
    fn positive_is_ok() {
        assert_eq!(check(0), Ok(0));
        assert_eq!(check(5), Ok(5));
    }

    #[test]
    fn round_trip() {
        for n in 1..150 {
            assert_eq!(code(from_code(n)), n);
        }
    }
}
