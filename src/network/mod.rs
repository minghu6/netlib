use std::net::Ipv4Addr;

use crate::aux::From2;

pub mod ip;
pub mod icmp;
mod icmp_spec;

use libc::in_addr;

impl From2<in_addr> for Ipv4Addr {
    fn from2(val: in_addr) -> Self {
        Ipv4Addr::from(val.s_addr)
    }
}

