use std::{
    ffi::CStr,
    mem::{size_of, zeroed},
    net::{Ipv4Addr, Ipv6Addr},
    ptr::null_mut,
};

pub mod icmp;
mod icmp_spec;
pub mod ip;
pub mod arp;


use libc::{
    freeifaddrs, getifaddrs as cgetifaddrs, getsockname, sockaddr_in,
    sockaddr_in6, socklen_t, sockaddr,
};

use crate::{
    aux::ntohl,
    data::{rtnl_link_stats, SAFamily, SockAddrIn},
    error::*,
    throw_errno,
};


/// Based from [rfc1071](https://www.rfc-editor.org/rfc/inline-errata/rfc1071.html)
pub unsafe fn inet_cksum(mut data: *const u8, mut len: usize) -> u16 {
    // let mut data = data;
    // let mut len = len;

    // the len should bo more than u16::MAX,
    // so u32 should be able to hold the sum.
    let mut sum = 0u32;
    let is_odd = len & 0x1 > 0;

    // sum every two bytes
    while len & 0xfffe > 0 {
        sum += *(data as *const u16) as u32;
        data = data.add(2);
        len -= 2;
    }

    // add the last one byte for odd len
    if is_odd {
        sum += (((*data as u16) << 8) & 0xff00) as u32;
    }

    // add extra overflow bits to the LSB (which diff from warpping_add)
    sum = (sum & 0xffff) + (sum >> 16);
    // at most overflow once
    sum += sum >> 16;

    // equal to while (sum >> 16) > 0 { sum = (sum & 0xffff) + (sum >> 16) }

    // as form of one's completion
    !(sum as u16)
}


pub unsafe fn getifaddrs() -> Result<IfAddrs> {
    let mut ifa = null_mut();

    throw_errno!(cgetifaddrs(&mut ifa) throws GetIfAddrs);

    let mut items = vec![];

    while !ifa.is_null() {
        if (*ifa).ifa_addr.is_null() {
            ifa = (*ifa).ifa_next;
            continue;
        }

        let family = (*(*ifa).ifa_addr).sa_family;
        let name =
            CStr::from_ptr((*ifa).ifa_name).to_str().unwrap().to_owned();

        let item = if family == SAFamily::Inet as u16 {
            IfAddrItem::Inet {
                name,
                addr: Ipv4Addr::from(ntohl(
                    (*((*ifa).ifa_addr as *mut sockaddr_in)).sin_addr.s_addr,
                )),
                mask: Ipv4Addr::from(ntohl(
                    (*((*ifa).ifa_netmask as *mut sockaddr_in))
                        .sin_addr
                        .s_addr,
                )),
            }
        }
        else if family == SAFamily::Inet6 as u16 {
            IfAddrItem::Inet6 {
                name,
                addr: Ipv6Addr::from(
                    (*((*ifa).ifa_addr as *mut sockaddr_in6))
                        .sin6_addr
                        .s6_addr,
                ),
                mask: Ipv6Addr::from(
                    (*((*ifa).ifa_netmask as *mut sockaddr_in6))
                        .sin6_addr
                        .s6_addr,
                ),
            }
        }
        else if family == SAFamily::Packet as u16
            && !(*ifa).ifa_data.is_null()
        {
            IfAddrItem::Packet {
                name,
                stats: *((*ifa).ifa_data as *const rtnl_link_stats),
            }
        }
        else {
            unimplemented!()
        };

        items.push(item);

        ifa = (*ifa).ifa_next;
    }

    freeifaddrs(ifa);

    Ok(IfAddrs(items))
}


pub unsafe fn getsockname_sockaddr_in(socket: i32) -> Result<SockAddrIn> {
    let mut addr = zeroed::<sockaddr_in>();
    let mut addr_len = size_of::<sockaddr_in>() as socklen_t;

    throw_errno!(getsockname(
        socket,
        &mut addr as *mut sockaddr_in as *mut sockaddr,
        &mut addr_len as *mut socklen_t
    ) throws GetSockName );

    Ok(SockAddrIn::from(addr))
}


#[derive(Debug)]
#[repr(transparent)]
pub struct IfAddrs(Vec<IfAddrItem>);

/// Support AF_INET, AF_INET6, AF_PACKET
#[derive(Debug)]
pub enum IfAddrItem {
    Inet {
        name: String,
        addr: Ipv4Addr,
        mask: Ipv4Addr,
    },
    Inet6 {
        name: String,
        addr: Ipv6Addr,
        mask: Ipv6Addr,
    },
    #[cfg(target_os = "linux")]
    Packet {
        name: String,
        stats: rtnl_link_stats,
    },
}


impl IfAddrs {
    /// Get first sockaddr_in from list (exclude 127.0.0.1)
    pub fn get_sockaddr_in(&self) -> Option<SockAddrIn> {
        for item in self.0.iter() {
            if let IfAddrItem::Inet {
                name: _,
                addr,
                mask: _,
            } = item
            {
                if !addr.is_loopback() {
                    return Some(SockAddrIn::from(*addr));
                }
            }
        }

        None
    }
}


#[allow(unused)]
#[cfg(test)]
mod tests {
    use std::mem::transmute;

    use super::{getifaddrs, IfAddrItem, IfAddrs};
    use crate::{aux::random_u8, network::inet_cksum};

    const RANDOM_BYTES_LEN: usize = 10000;

    fn random_big_bytes(buf: &mut [u8]) {
        for i in 0..RANDOM_BYTES_LEN {
            buf[i] = random_u8();
        }
    }

    // unsafe fn inet_cksum2(mut data: *const u8, mut len: usize) -> u16 {
    //     let mut sum = 0u16;
    //     let is_odd = len & 0x1 > 0;

    //     while len & 0xfffe > 0 {
    //         sum = sum.wrapping_add(*(data as *const u16));
    //         data = data.add(2);
    //         len -= 2;
    //     }

    //     // add the last one byte for odd len
    //     if is_odd {
    //         sum = sum.wrapping_add(((*data as u16) << 8) & 0xff00);
    //     }

    //     !sum
    // }

    // #[test]
    // fn test_cksum() {
    //     unsafe {
    //         let mut buf = [0u8; RANDOM_BYTES_LEN];

    //         for i in 0..100 {
    //             random_big_bytes(&mut buf);
    //             let cksum = inet_cksum(buf.as_ptr(), buf.len());
    //             let cksum2 = inet_cksum2(buf.as_ptr(), buf.len());

    //             assert_eq!(cksum, cksum2, "{i}:")
    //         }
    //     }
    // }


    #[test]
    fn test_getifaddrs() {
        unsafe {
            let ifaddrs = getifaddrs().unwrap();
            let items = transmute::<IfAddrs, Vec<IfAddrItem>>(ifaddrs);

            for item in &items[..] {
                println!("{:#?}", item);
            }

            let ifaddrs = transmute::<Vec<IfAddrItem>, IfAddrs>(items);

            println!("addr: {:?}", ifaddrs.get_sockaddr_in().unwrap())
        }
    }
}
