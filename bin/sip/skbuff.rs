use std::{ptr::drop_in_place, mem::zeroed, alloc::{alloc_zeroed, Layout}};

use netlib::{
    datalink::Eth,
    network::{arp::ARP, icmp::ICMP, ip::{IP, Protocol, ToS}},
    transport::{tcp::TCP, udp::UDP}, defraw1, data::InAddrN, view::U16N, alignsz, defraw0,
};

use libc::sem_t;

use crate::eth::NetDevice;


////////////////////////////////////////////////////////////////////////////////
//// Structure

defraw1! {
    pub union THdr {
        tcph: *mut TCP,
        udph: *mut UDP,
        icmph: *mut ICMP,
        raw: *mut u8,
    }

    pub union NetHdr {
        iph: *mut IP,
        arph: *mut ARP,
        raw: *mut u8,
    }

    pub union PhyHdr {
        ethh: *mut Eth,
        raw: *mut u8,
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

defraw0! {
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
        /// 当前网络层的尾部
        tail: *mut u8,
        /// 实际网络层的尾部
        end: *mut u8
    }
}



////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl SKBuff {
    /// cap: Eth frame cap
    pub fn with_capcity(cap: usize) -> Self {
        unsafe {
            let mut it: Self = zeroed();

            let sz = alignsz!(cap, 4);
            it.head = alloc_zeroed(Layout::array::<u8>(sz).unwrap());
            it.end = it.head.add(sz);
            it.data = it.head;
            it.tail = it.data;

            it.total_len = cap as u32;
            it.curproto_len = cap as u32;

            it
        }
    }

    pub fn forward(&mut self, dist: usize) -> *mut u8 {
        unsafe {
            let tmp = self.tail;
            self.tail = self.tail.add(dist);

            // if self.curproto_len < dist as u32 {
            //     eprintln!("")
            // }

            self.curproto_len -= dist as u32;
            tmp
        }
    }
}


impl Drop for SKBuff {
    fn drop(&mut self) {
        unsafe { drop_in_place(self.head) }
    }
}

