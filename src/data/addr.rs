////////////////////////////////////////////////////////////////////////////////
//// Data Structures

use std::{fmt::Debug, mem::transmute, net::Ipv4Addr};

use default_net::Gateway;
use libc::sockaddr_in;

use crate::{
    aux::{htonl, ntohl},
    datalink::{EthTypeN, PacType},
    defraw, deftransparent,
    network::arp::ARPHT,
    view::U16N, error::{ NetErr, Result }, s,
};


////////////////////////////////////////////////////////////////////////////////
//// Trait


pub trait Subnet {
    fn subnet(&self, mask: &Self) -> Self;
    fn broadcast(&self, mask: &Self) -> Self;
}


////////////////////////////////////////////////////////////////////////////////
//// Structure

defraw! {
    #[repr(u16)]
    /// Some field has been elimited, from x86_64 linux gnu
    pub enum SAFamily {
        /// AF_UNSPEC 0
        #[default]
        UnSpec,
        /// AF_LOCAL (including synonym AF_UNIX, AF_FILE) 1
        Local = 1,
        /// AF_INET 2 (sockaddr_in, ipv4)
        Inet = 2,
        /// AF_INET 10
        Inet6 = 10,
        /// AF_PACKET 17 (rx/tx raw packets at the Layer 2)
        Packet = 17,
    }

    /// Synonym libc::sockaddr_in
    pub struct SockAddrIn {
        family: SAFamily,
        port: U16N,
        /// IPv4 Address
        addr: InAddrN,
        zero_pading: [u8; 8],
    }

    pub struct SockAddrLL {
        family: u16,
        proto: EthTypeN,
        ifindex: i32,
        hatype: ARPHT,
        pkttype: PacType,
        halen: u8,
        addr: [u8; 8]
    }

}



deftransparent! {
    /// Network bytes order
    pub struct InAddrN(u32);
}


// /// Native bytes order
// pub struct InAddr(pub u32);


////////////////////////////////////////////////////////////////////////////////
//// Implementation


impl Into<Ipv4Addr> for InAddrN {
    fn into(self) -> Ipv4Addr {
        Ipv4Addr::from(unsafe { ntohl(self.0) })
    }
}

impl Debug for InAddrN {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let addr: Ipv4Addr = (*self).into();
        write!(f, "{addr:?}")
    }
}

impl Subnet for InAddrN {
    fn subnet(&self, mask: &Self) -> Self {
        Self(self.0 | !mask.0)
    }

    fn broadcast(&self, mask: &Self) -> Self {
        Self(self.0 & mask.0)
    }
}

impl InAddrN {
    pub fn from_native_u32(v: u32) -> Self {
        Self(unsafe { htonl(v) })
    }

    pub fn from_native_sockaddr_in(v: sockaddr_in) -> Self {
        Self::from_native_u32(v.sin_addr.s_addr)
    }

    pub fn from_ipv4addr(addr: Ipv4Addr) -> Self {
        Self::from_native_u32(addr.into())
    }

    pub fn native(self) -> u32 {
        unsafe { ntohl(self.0) }
    }

    pub fn ipv4(self) -> Ipv4Addr {
        Ipv4Addr::from(self.native())
    }
}


impl From<Ipv4Addr> for SockAddrIn {
    fn from(ipv4: Ipv4Addr) -> Self {
        Self {
            family: SAFamily::Inet,
            port: U16N::default(),
            addr: InAddrN::from_ipv4addr(ipv4),
            zero_pading: [0; 8],
        }
    }
}

impl Into<sockaddr_in> for SockAddrIn {
    fn into(self) -> sockaddr_in {
        unsafe { transmute(self) }
    }
}

impl From<sockaddr_in> for SockAddrIn {
    fn from(addr: sockaddr_in) -> Self {
        unsafe { transmute(addr) }
    }
}


impl Subnet for Ipv4Addr {
    fn subnet(&self, mask: &Self) -> Self {
        let addru: u32 = self.clone().into();
        let masku: u32 = mask.clone().into();

        Ipv4Addr::from(addru & masku)
    }

    fn broadcast(&self, mask: &Self) -> Self {
        let addru: u32 = self.clone().into();
        let masku: u32 = mask.clone().into();

        Ipv4Addr::from(addru | !masku)
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Function

pub fn getgateway() -> Result<Gateway> {
    match default_net::get_default_interface() {
        Ok(default_interface) => {
            default_interface.gateway.ok_or(NetErr::GetGateway(s!("")))
        },
        Err(msg) => Err(NetErr::GetGateway(msg))
    }
}



#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    use std::{
        ffi::{CStr, CString},
        mem::size_of,
        mem::transmute,
        net::Ipv4Addr,
    };

    use libc::{
        c_int, c_void, free, hostent, sockaddr_in, socklen_t, AF_INET,
    };

    use crate::{
        aux::{htonl, inet_addr, inet_ntoa},
        data::{SAFamily, SockAddrIn},
    };

    #[allow(unused)]
    extern "C" {
        fn gethostbyaddr(
            addr: *const c_void,
            len: socklen_t,
            ty: c_int,
        ) -> *mut hostent;
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
                sin_addr: libc::in_addr {
                    s_addr: htonl(ipv4.into()),
                },
                sin_zero: [0; 8],
            };

            assert_eq!(sockaddr1, sockaddr2);

            let caddrs_ptr = inet_ntoa(libc::in_addr {
                s_addr: ipv4.into(),
            });
            let caddrs = CStr::from_ptr(caddrs_ptr);
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
