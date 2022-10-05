use libc::__errno_location;




/// Socket Error Number
///
/// `LANGUAGE=en_US:en errno -l`

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
#[non_exhaustive]
pub enum ErrNo {
    UnknownErrNo,
    /// 1 - Not owner
    EPERM,
    /// 2 - No such file
    ENOENT,
    /// 3 -
    ESRCH,
    /// 4 -
    EINTR,
    /// 5 -
    EIO,
    /// 6 -
    ENXIO,
    /// 7 -
    E2BIG,
    /// 8 -
    ENOEXEC,
    /// 9 -
    EBADF,
    /// 10 -
    ECHILD,
    /// 11 - Resource temporarily unavailable (EWOULDBLOCK)
    EAGAIN,
    /// 12 -
    ENOMEM,
    /// 13 -
    EACCES,
    /// 14 -
    EFAULT,
    /// 15 -
    ENOTBLK,
    /// 16
    EBUSY,
    /// 17
    EEXIST,
    /// 18
    EXDEV,
    /// 19
    ENODEV,
    /// 20
    ENOTDIR,
    /// 21
    EISDIR,
    /// 22 - Invalid Parameter
    EINVAL,
    /// 23
    ENFILE,
    /// 24
    EMFILE,
    /// 25
    ENOTTY,
    /// 26
    ETXTBSY,
    /// 27
    EFBIG,
    /// 28
    ENOSPC,
    /// 29
    ESPIPE,
    /// 30
    EROFS,
    /// 31
    EMLINK,
    /// 32
    EPIPE,
    /// 33
    EDOM,
    /// 34
    ERANGE,
    /// 35 - Resource deadlock avoided (EDEADLOCK)
    EDEADLK,
    /// 36
    ENAMETOOLONG,
    /// 37 - No Avaliable Lock
    ENOLCK,
    /// 38
    ENOSYS,
    /// 39
    ENOTEMPTY,
    /// 40
    ELOOP,
    /// 42
    ENOMSG = 42,
    /// 43
    EIDRM,
    /// 44
    ECHRNG,
    /// 45
    EL2NSYNC,
    /// 46
    EL3HLT,
    /// 47
    EL3RST,
    /// 48
    ELNRNG,
    /// 49
    EUNATCH,
    /// 50
    ENOCSI,
    /// 51
    EL2HLT,
    /// 52
    EBADE,
    /// 53
    EBADR,
    /// 54
    EXFULL,
    /// 55
    ENOANO,
    /// 56
    EBADRQC,
    /// 57
    EBADSLT,
    /// 59 - Bad font file format
    EBFONT = 59,
    /// 60
    ENOSTR,
    /// 61
    ENODATA,
    /// 62
    ETIME,
    /// 63
    ENOSR,
    /// 64
    ENONET,
    /// 65
    ENOPKG,
    /// 66
    EREMOTE,
    /// 67
    ENOLINK,
    /// 68
    EADV,
    /// 69
    ESRMNT,
    /// 70
    ECOMM,
    /// 71
    EPROTO,
    /// 72
    EMULTIHOP,
    /// 73
    EDOTDOT,
    /// 74
    EBADMSG,
    /// 75
    EOVERFLOW,
    /// 76
    ENOTUNIQ,
    /// 77
    EBADFD,
    /// 78
    EREMCHG,
    /// 79
    ELIBACC,
    /// 80
    ELIBBAD,
    /// 81
    ELIBSCN,
    /// 82
    ELIBMAX,
    /// 83
    ELIBEXEC,
    /// 84
    EILSEQ,
    /// 85
    ERESTART,
    /// 86
    ESTRPIPE,
    /// 87
    EUSERS,
    /// 88
    ENOTSOCK,
    /// 89
    EDESTADDRREQ,
    /// 90
    EMSGSIZE,
    /// 91
    EPROTOTYPE,
    /// 92
    ENOPROTOOPT,
    /// 93
    EPROTONOSUPPORT,
    /// 94
    ESOCKTNOSUPPORT,
    /// 95 - Operation not supported (ENOTSUP)
    EOPNOTSUPP,
    /// 96
    EPFNOSUPPORT,
    /// 97
    EAFNOSUPPORT,
    /// 98
    EADDRINUSE,
    /// 99
    EADDRNOTAVAIL,
    /// 100
    ENETDOWN,
    /// 101
    ENETUNREACH,
    /// 102
    ENETRESET,
    /// 103
    ECONNABORTED,
    /// 104
    ECONNRESET,
    /// 105
    ENOBUFS,
    /// 106
    EISCONN,
    /// 107
    ENOTCONN,
    /// 108
    ESHUTDOWN,
    /// 109
    ETOOMANYREFS,
    /// 110
    ETIMEDOUT,
    /// 111
    ECONNREFUSED,
    /// 112
    EHOSTDOWN,
    /// 113
    EHOSTUNREACH,
    /// 114
    EALREADY,
    /// 115
    EINPROGRESS,
    /// 116
    ESTALE,
    /// 117
    EUCLEAN,
    /// 118
    ENOTNAM,
    /// 119
    ENAVAIL,
    /// 120
    EISNAM,
    /// 121
    EREMOTEIO,
    /// 122
    EDQUOT,
    /// 123
    ENOMEDIUM,
    /// 124
    EMEDIUMTYPE,
    /// 125
    ECANCELED,
    /// 126
    ENOKEY,
    /// 127
    EKEYEXPIRED,
    /// 128
    EKEYREVOKED,
    /// 129
    EKEYREJECTED,
    /// 130
    EOWNERDEAD,
    /// 131
    ENOTRECOVERABLE,
    /// 132
    ERFKILL,
    /// 133
    EHWPOISON,
}

impl From<i32> for ErrNo {
    fn from(val: i32) -> Self {
        unsafe {
            std::mem::transmute(val)
        }
    }
}

impl ErrNo {
    pub fn fetch() -> Self {
        let errno = unsafe { *__errno_location() };
        Self::from(errno)
    }
}
