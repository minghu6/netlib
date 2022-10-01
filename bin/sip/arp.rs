use std::{
    cell::RefCell,
    mem::{size_of, zeroed},
};

use libc::{time, time_t, ETH_ALEN, ETH_ZLEN};
use log::info;
use netlib::{
    data::{InAddrN, Subnet},
    datalink::{Eth, EthTypeE, Mac},
    defraw1, deftransparent1,
    error::NetErr,
    network::arp::{ARPOpE, ARP, ARPHTE},
    Result,
};

use crate::{eth::NetDevice, skbuff::SKBuff};



////////////////////////////////////////////////////////////////////////////////
//// Constant

pub const ARPTABSZ: usize = 10;
pub const ARPLIVE: usize = 10 * 60;


////////////////////////////////////////////////////////////////////////////////
//// Structure

defraw1! {
    /// Using TRLU replace policy
    pub struct ARPRecod {
        /// 0 means empty
        is_valid: bool,
        age: u16,
        ip: InAddrN,
        /// Last create time (i64/i32)
        ctime: time_t,
        mac: Mac,
    }

    pub struct ARPTab {
        tab: [ARPRecod; ARPTABSZ],
    }

}

deftransparent1! {}

thread_local! {
    pub static ARPTAB: RefCell<ARPTab> = RefCell::new(unsafe { zeroed() });

}


////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl ARPTab {
    pub fn get_mut_and_update(
        &mut self,
        ip: InAddrN,
        live: time_t,
    ) -> Option<&mut ARPRecod> {
        let now = unsafe { time(0 as _) };

        for rec in self.tab.iter_mut().filter(|rec| rec.is_valid) {
            if rec.ctime + live < now {
                rec.is_valid = false;
                continue;
            }

            if rec.ip == ip {
                rec.age = 0;
                return Some(rec);
            }

            if rec.age < u16::MAX {
                rec.age += 1;
            }
        }

        None
    }

    /// Replace the smallest ctime if it's empty
    pub fn insert(&mut self, ip: InAddrN, mac: Mac, live: time_t) {
        let now = unsafe { time(0 as _) };

        if let Some(rec) = self.get_mut_and_update(ip, live) {
            debug_assert!(rec.is_valid);
            rec.mac = mac;
            rec.ctime = now;
            return;
        }

        let rec =
            if let Some(rec) = self.tab.iter_mut().find(|rec| !rec.is_valid) {
                rec
            }
            else {
                self.tab.iter_mut().max_by_key(|rec| rec.age).unwrap()
            };

        rec.is_valid = true;
        rec.age = 0;
        rec.ip = ip;
        rec.ctime = now;
        rec.mac = mac;
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Function

pub unsafe fn arp_create(
    dev: &NetDevice,
    op: ARPOpE,
    src_ip: InAddrN,
    dst_ip: InAddrN,
    mut src_mac: Mac,
    mut dst_mac: Mac,
    target_mac: Mac,
) -> SKBuff {
    let mut skb = SKBuff::with_capcity(ETH_ZLEN as usize);

    skb.phy.raw = skb.forward(size_of::<Eth>());
    skb.nh.raw = skb.forward(size_of::<ARP>());
    skb.dev = dev;

    if src_mac.is_empty() {
        src_mac = dev.hwa;
    }
    if dst_mac.is_empty() {
        dst_mac = dev.hwa_broadcast;
    }

    let ethh = &mut (*skb.phy.ethh);
    ethh.proto = EthTypeE::ARP.net();
    ethh.dst = dst_mac;
    ethh.src = src_mac;

    let arph = &mut *skb.nh.arph;

    arph.op = op.net();
    arph.hrd = ARPHTE::Ethernet10Mb.net();
    arph.proto = EthTypeE::IPv4.net();
    arph.hln = ETH_ALEN as u8;
    arph.pln = 4;

    arph.sha = src_mac;
    arph.sip = src_ip;
    arph.tip = dst_ip;

    if !target_mac.is_empty() {
        arph.tha = target_mac;
    }
    else {
        arph.tha.clear();
    }

    skb
}


pub unsafe fn arp_send(
    dev: &NetDevice,
    op: ARPOpE,
    src_ip: InAddrN,
    dst_ip: InAddrN,
    src_mac: Mac,
    dst_mac: Mac,
    target_mac: Mac,
) -> Result<()> {
    let skb =
        arp_create(dev, op, src_ip, dst_ip, src_mac, dst_mac, target_mac);

    info!("Output: {:#?}", *skb.nh.arph);

    dev.linkoutput(&skb)?;

    Ok(())
}


pub unsafe fn arp_req(dev: &NetDevice, ip: InAddrN) -> Result<()> {
    let mask = dev.ip_netmask.ipv4();
    let tip = if ip.ipv4().subnet(&mask) == dev.ip_host.ipv4().subnet(&mask) {
        ip
    }
    else {
        dev.ip_gateway
    };

    arp_send(
        dev,
        ARPOpE::Request,
        dev.ip_host,
        tip,
        dev.hwa,
        zeroed(),
        zeroed(),
    )?;

    Ok(())
}


pub unsafe fn arp_input(dev: &NetDevice, skb: SKBuff) -> Result<()> {
    if (skb.total_len as usize) < size_of::<ARP>() {
        return Err(NetErr::AnyWay(format!(
            "ARP Too short {}, expect {}",
            skb.total_len,
            size_of::<ARP>()
        )));
    }

    let arph = &*skb.nh.arph;
    let arptip = arph.tip;
    if arptip == dev.ip_host {
        ARPTAB.with_borrow_mut(|tab|
            tab.insert(arptip, dev.hwa, ARPLIVE as i64)
        )
    }

    let arphop: ARPOpE = arph.op.native()?;
    let arphsip = arph.sip;
    let arphsha = arph.sha;
    let eth = &*skb.phy.ethh;

    match arphop {
        ARPOpE::Request => {
            arp_send(
                dev,
                ARPOpE::Reply,
                dev.ip_host,
                arphsip,
                dev.hwa,
                eth.src,
                arphsha
            )?;
        },
        ARPOpE::Reply => {
        }
    }

    ARPTAB.with_borrow_mut(|tab|
        tab.insert(arphsip, eth.src, ARPLIVE as i64)
    );


    Ok(())
}
