#![feature(box_syntax)]

use std::{
    error::Error,
    mem::{size_of, zeroed},
    net::Ipv4Addr,
    ptr::null_mut,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, sleep},
    time::{Duration, Instant},
};

use bincode::{options, Options};
use clap::Parser;
use libc::{
    __errno_location, c_void, fd_set, getpid, recv, select, sendto,
    setsockopt, sockaddr, sockaddr_in, socket, timeval, AF_INET, AF_PACKET,
    EINTR, FD_SET, FD_ZERO, IPPROTO_IP, IP_MULTICAST_IF, IP_MULTICAST_TTL,
    IP_TTL, SOCK_RAW, SOL_SOCKET, SO_BROADCAST, SO_RCVBUF,
};
use netlib::{
    aux::HostOrIPv4,
    bincode_options,
    data::{SockAddrIn, SockAddrLL, InAddrN},
    datalink::{EthTypeE, EthTypeN, PacType, Mac, Eth},
    defe,
    network::{
        arp::{ARPHTE, ARP, ARPOp, ARPOpE},
        icmp::{ICMPType, ICMP},
        inet_cksum,
        ip::{Protocol, IP},
    }, error::{ NetErr, Result }, throw_errno,
};
use signal_hook::consts;
use signal_hook::flag::register;


const BUF_SIZE: usize = 60;


#[derive(Parser)]
#[clap()]
struct Cli {
    #[clap()]
    dst: String,
}


unsafe fn send_arp(sock: i32, ifindex: i32, src_mac: Mac, src_ip: InAddrN, dst_ip: InAddrN) -> Result<isize> {
    let sockaddr = SockAddrLL {
        family: AF_PACKET as u16,
        proto: EthTypeE::ARP.net(),
        ifindex,
        hatype: ARPHTE::Ethernet10Mb.net(),
        pkttype: PacType::Broadcast,
        halen: size_of::<Mac>() as u8,
        addr: src_mac.into_arr8(),
    };

    let mut buf = [0u8; BUF_SIZE];

    /* Init package */
    let eth = Eth {
        dst: Mac::broadcast(),
        src: src_mac,
        proto: EthTypeE::ARP.net(),
    };
    let arp = ARP {
        hrd: ARPHTE::Ethernet10Mb.net(),
        proto: EthTypeE::IPv4.net(),
        hln: size_of::<Mac>() as u8,
        pln: size_of::<InAddrN> as u8,
        op: ARPOpE::Request.net(),
        sha: src_mac,
        sip: src_ip,
        tha: Mac::default(),
        tip: dst_ip,
    };

    write!(&mut buf, eth);

    Ok(throw_errno!(
        sendto(
            sock,
            buf.as_ptr() as *const c_void,
            42,
            0,
            &sockaddr as *const SockAddrLL as *const sockaddr,
            size_of::<SockAddrLL>() as u32
        ) throws SendTo
    ))
}


fn main() -> std::result::Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let dst = cli.dst;


    Ok(())
}
