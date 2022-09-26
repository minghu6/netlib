use std::{
    ffi::CStr,
    net::{Ipv4Addr, Ipv6Addr},
    ptr::null_mut,
};

use ifstructs::ifreq;
use libc::{
    freeifaddrs, getifaddrs as cgetifaddrs, ioctl, sockaddr_in, sockaddr_in6,
    socket, AF_PACKET, SOCK_RAW
};

use crate::{
    aux::ntohl,
    data::{rtnl_link_stats, SAFamily, SockAddrIn},
    error::Result,
    throw_errno, datalink::{EthTypeE, Mac},
};


////////////////////////////////////////////////////////////////////////////////
//// Constant

/* SIOC G(et) IF INDEX */
pub const SIOCGIFINDEX: u64 = 0x8933;
pub const SIOCGIFHWADDR: u64 = 0x8933;



////////////////////////////////////////////////////////////////////////////////
//// Structure

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


////////////////////////////////////////////////////////////////////////////////
//// Implementation

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

    pub fn get_inet_items(&self) -> impl Iterator<Item=(&str, &Ipv4Addr, &Ipv4Addr)> {
        let mut iter = self.0.iter();

        std::iter::from_fn(move || {
            loop {
                if let Some(item) = iter.next() {
                    if let IfAddrItem::Inet { name, addr, mask }
                    = item {
                        return Some((name.as_str(), addr, mask));
                    }
                }
                else {
                    break;
                }
            }

            None
        })
    }

}



////////////////////////////////////////////////////////////////////////////////
//// Function

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


pub unsafe fn getifnth(ifname: &str) -> Option<i32> {
    let sock = socket(AF_PACKET, SOCK_RAW, EthTypeE::AVTP.net().val() as i32);

    if sock <= 0 {
        return None;
    }

    let mut ifr = if let Ok(ifr) = ifreq::from_name(ifname) {
        ifr
    }
    else {
        return None;
    };

    if ioctl(sock, SIOCGIFINDEX, &mut ifr) == -1 {
        return None;
    }

    Some(ifr.ifr_ifru.ifr_ifindex)
}

pub unsafe fn getifmac(ifname: &str) -> Option<Mac> {
    let sock = socket(AF_PACKET, SOCK_RAW, EthTypeE::AVTP.net().val() as i32);

    if sock <= 0 {
        return None;
    }

    let mut ifr = if let Ok(ifr) = ifreq::from_name(ifname) {
        ifr
    }
    else {
        return None;
    };

    if ioctl(sock, SIOCGIFHWADDR, &mut ifr) == -1 {
        return None;
    }

    let mac = Mac::from_slice(
        &ifr.ifr_ifru.ifr_hwaddr.sa_data
    );

    Some(mac)
}



#[cfg(test)]
mod tests {
    use std::mem::transmute;

    use crate::network::if_::getifnth;

    use super::{getifaddrs, IfAddrItem, IfAddrs};

    #[test]
    fn test_getifaddrs() {
        unsafe {
            let ifaddrs = getifaddrs().unwrap();
            let items = transmute::<IfAddrs, Vec<IfAddrItem>>(ifaddrs);

            for item in &items[..] {
                println!("{:#?}", item);
            }

            let ifaddrs = transmute::<Vec<IfAddrItem>, IfAddrs>(items);

            println!("addr: {:?}", ifaddrs.get_sockaddr_in().unwrap());

            let ifnth =  getifnth("wlp3s0");

            println!("{ifnth:?}");
        }
    }
}
