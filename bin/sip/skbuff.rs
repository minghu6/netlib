use netlib::{
    datalink::Eth,
    network::{arp::ARP, icmp::ICMP, ip::{IP, Protocol, ToS}},
    transport::{tcp::TCP, udp::UDP}, defraw0, data::InAddrN, view::U16N,
};

use libc::sem_t;

use crate::eth::NetDevice;


defraw0! {
    pub union THdr {
        tcphdr: *mut TCP,
        udphdr: *mut UDP,
        icmphdr: *mut ICMP,
        raw: *mut u8,
    }

    pub union NetHdr {
        iphdr: *mut IP,
        arphdr: *mut ARP,
        raw: *mut u8,
    }

    pub union PhyHdr {
        ethhdr: *mut Eth,
        raw: *mut u8,
    }

    pub struct SKBuff {
        next: *mut SKBuff,
        th: THdr,
        nh: NetHdr,
        phy: PhyHdr,
        dev: *const NetDevice,
        proto: Protocol,
        total_len: u32,
        curproto_len: u32,
        csum: u8,
        /// If IP header has been checked (cksum)
        ip_checked: bool,
        /// 实际网络层的头部
        head: *mut u8,
        /// 当前网络层的头部
        data: *mut u8,
        /// 当前网络层的头部
        tail: *mut u8,
        /// 实际网络层的尾部
        end: *mut u8
    }

    pub struct PCBIP {
        src_ip: InAddrN,
        dst_ip: InAddrN,
        so_opt: u16,
        tos: ToS,
        ttl: u8,
        /// link layer resp;ition hint
        addrhint: u8
    }

    pub struct PCBUDP {
        src_ip: InAddrN,
        src_port: U16N,
        dst_ip: InAddrN,
        dst_port: U16N,
        tos: ToS,
        ttl: u8,
        flags: u8,
    }

    pub union PCB {
        ip: *mut PCBIP,
        udp: *mut PCBUDP
    }

    struct Sock {
        type_: i32,
        state: i32,
        pcb: PCB,
        err: i32,
        skb_recv: *mut SKBuff,

        /// semaphore of receive buffer
        sem_recv: sem_t,
        sd: i32,
        timeout_recv: i32,
        avail_recv: u16
    }
}

