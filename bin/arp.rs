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
    __errno_location, c_void, fd_set, getpid, recv, select, sendto, setsockopt,
    sockaddr, sockaddr_in, socket, timeval, AF_INET, EINTR, FD_SET, FD_ZERO,
    IPPROTO_IP, IP_MULTICAST_IF, IP_MULTICAST_TTL, IP_TTL, SOCK_RAW, SOL_SOCKET, SO_BROADCAST, SO_RCVBUF,
};
use netlib::{
    aux::HostOrIPv4,
    bincode_options,
    data::SockAddrIn,
    defe,
    network::{
        icmp::{ICMPType, ICMP},
        inet_cksum,
        ip::{Protocol, IP},
    },
};
use signal_hook::consts;
use signal_hook::flag::register;



#[derive(Parser)]
#[clap()]
struct Cli {
    #[clap()]
    dst: String,
}


fn send_arp() {


}


fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let dst = cli.dst;


    Ok(())
}
