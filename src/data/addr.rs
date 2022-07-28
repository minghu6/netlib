////////////////////////////////////////////////////////////////////////////////
//// Data Structures

use std::net::Ipv4Addr;

use crate::aux::{htonl, ntohl, ntohs};

/// Synonym libc::sockaddr_in
#[repr(C)]
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct SockAddrIn {
    family: SAFamily,
    port: u16,
    /// IPv4 Address
    addr: u32,
    zero_pading: [u8; 8],
}


/// Some field has been elimited, from x86_64 linux gnu
#[repr(u16)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum SAFamily {
    /// AF_UNSPEC 0
    UnSpec,
    /// AF_LOCAL (including synonym AF_UNIX, AF_FILE) 1
    Local = 1,
    /// AF_INET 2 (sockaddr_in, ipv4)
    Inet = 2,
    /// AF_INET 10
    Inet6 = 10,
}


////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl From<Ipv4Addr> for SockAddrIn {
    fn from(ipv4: Ipv4Addr) -> Self {
        Self {
            family: SAFamily::Inet,
            port: 0,
            addr: unsafe { htonl(ipv4.into()) },
            zero_pading: [0; 8],
        }
    }
}

impl std::fmt::Debug for SockAddrIn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_struct("SockAddrIn")
            .field("family", &self.family)
            .field("port", &ntohs(self.port))
            .field("addr", &Ipv4Addr::from(ntohl(self.addr)))
            .finish()
        }
    }
}


#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    use std::{mem::size_of, net::Ipv4Addr, mem::transmute, ffi::{CString, CStr}};

    use libc::{c_void, socklen_t, c_int, hostent, sockaddr_in, AF_INET, free};

    use crate::{data::{SockAddrIn, SAFamily}, aux::{inet_addr, inet_ntoa, htonl}};

    #[allow(unused)]
    extern "C" {
        fn gethostbyaddr(addr: *const c_void, len: socklen_t, ty: c_int) -> *mut hostent;
        fn __h_errno_location() -> *mut c_int;
    }

    #[test]
    fn test_info_addr() {
        println!("sizeof struct in_addr: {}", size_of::<libc::in_addr>());

        unsafe {
            // let ipv4 = Ipv4Addr::new(39, 156, 66, 10);
            let ipv4 = Ipv4Addr::new(127, 0, 0, 1);

            let w_scokaddrin = SockAddrIn::from(ipv4);
            let sockaddr1: sockaddr_in = transmute(w_scokaddrin);

            let sockaddr2 = sockaddr_in {
                sin_family: AF_INET as u16,
                sin_port: 0,
                sin_addr: libc::in_addr { s_addr: htonl(ipv4.into()) },
                sin_zero: [0; 8],
            };

            assert_eq!(sockaddr1, sockaddr2);

            let caddrs_ptr = inet_ntoa(libc::in_addr { s_addr: ipv4.into() });
            let caddrs = CStr::from_ptr(
                caddrs_ptr
            );
            let addrs = caddrs.to_str().unwrap();
            println!("{}", addrs);

            // free(caddrs_ptr as *mut c_void);

            // let htent = gethostbyaddr(
            //     &sockaddr2 as *const sockaddr_in as *const c_void,
            //     size_of::<sockaddr_in>() as u32,
            //     AF_INET
            // );

            // if htent.is_null() {
            //     eprintln!("htent is null");
            //     let herrno = *__h_errno_location();
            //     // # define HOST_NOT_FOUND	1	/* Authoritative Answer Host not found.  */
            //     // # define TRY_AGAIN	2	/* Non-Authoritative Host not found,
            //     //                 or SERVERFAIL.  */
            //     // # define NO_RECOVERY	3	/* Non recoverable errors, FORMERR, REFUSED,
            //     //                 NOTIMP.  */
            //     // # define NO_DATA	4
            //     eprintln!("herrno: {}", herrno);
            // }
            // else {
            //     println!("{:#?}", &*htent)
            // }

        }

    }
}
