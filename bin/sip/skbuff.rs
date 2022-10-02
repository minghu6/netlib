use std::{
    alloc::{alloc_zeroed, Layout},
    cell::RefCell,
    cmp::Ordering,
    collections::BTreeSet,
    mem::zeroed,
    ptr::drop_in_place,
};

use libc::sem_t;
use netlib::{
    alignsz,
    data::InAddrN,
    datalink::Eth,
    defraw0, defraw1,
    network::{
        arp::ARP,
        icmp::ICMP,
        ip::{Protocol, ToS, IP},
    },
    transport::{tcp::TCP, udp::UDP},
    view::U16N,
};

use crate::eth::NetDevice;


////////////////////////////////////////////////////////////////////////////////
//// Macro

#[macro_export]
macro_rules! push_skbuff {
    ($expr:expr) => {
        $crate::skbuff::SKBHOLDER.with_borrow_mut(|x|
            x.push_get_raw($expr)
        )
    };
}



////////////////////////////////////////////////////////////////////////////////
//// ThreadLocal

thread_local! {
    pub static SKBHOLDER: RefCell<SKBuffHolder> = RefCell::new(SKBuffHolder::new());
}


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

pub struct SKBuffHolder {
    data: BTreeSet<RefCell<SKBuff>>,
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

    // pub fn key(&self) -> usize {

    // }

}


impl Drop for SKBuff {
    fn drop(&mut self) {
        unsafe { drop_in_place(self.head) }
    }
}


impl PartialEq for SKBuff {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.phy.raw == other.phy.raw }
    }
}

impl Eq for SKBuff {}


impl PartialOrd for SKBuff {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        unsafe { self.phy.raw.partial_cmp(&other.phy.raw) }
    }
}

impl Ord for SKBuff {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}


impl SKBuffHolder {
    pub fn new() -> Self {
        Self {
            data: BTreeSet::new(),
        }
    }

    pub fn push_get_raw(&mut self, skbuff: SKBuff) -> *mut SKBuff {
        let cell = RefCell::new(skbuff);
        let p = cell.as_ptr();
        self.data.insert(cell);

        p
    }

}

