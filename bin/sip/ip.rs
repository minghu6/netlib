use std::{
    mem::size_of,
    ptr::null_mut,
};

use libc::memcpy;
use netlib::{
    aux::htons,
    data::InAddrN,
    error::NetErr,
    network::ip::{FragFlag, FragOff, Protocol, IP},
    Result,
};

use crate::{
    eth::{NetDevice, ETH_HLEN},
    skbuff::SKBuff,
};



////////////////////////////////////////////////////////////////////////////////
//// Constant

pub const IPHLEN: usize = size_of::<IP>();


////////////////////////////////////////////////////////////////////////////////
//// Structure

#[allow(unused)]
struct Reass {
    next: *mut Reass,
    skb: *mut SKBuff,
    iphdr: IP,
    datagram_len: u16,
    flags: u8,
    timer: u8,
}


////////////////////////////////////////////////////////////////////////////////
//// Implementation




////////////////////////////////////////////////////////////////////////////////
//// Function

unsafe fn cksum_standard(datap: *mut u8, mut len: u16) -> u16 {
    let mut acc: u32 = 0;
    let mut octp = datap;

    while len > 1 {
        /* declare first octet as most signification
         * thus assume network order, ignoring host order
         */
        let mut src = (*octp as u32) << 8;
        octp = octp.add(1);

        /* declare second octet as least signification */
        src |= *octp as u32;
        octp = octp.add(1);

        acc += src;
        len -= 2;
    }

    if len > 0 {
        acc += (*octp as u32) << 8;
    }

    /* add deferred carry bits */
    acc = (acc >> 16) + (acc & 0x0000_ffff);
    if (acc & 0xffff_0000) != 0 {
        acc = (acc >> 16) + (acc & 0x0000_ffff);
    }

    htons(acc as u16)
}


pub unsafe fn cksum(datap: *mut u8, len: u16) -> u16 {
    let mut acc = cksum_standard(datap, len) as u32;

    while (acc >> 16) != 0 {
        acc = (acc & 0xffff) + (acc >> 16);
    }

    !(acc & 0xffff) as u16
}


unsafe fn validate_ip(dev: &NetDevice, skb: &mut SKBuff) -> Result<()> {
    let iph = &*skb.nh.iph;

    if iph.ihl_v.get_version() != 4 {
        return Err(NetErr::AnyWay(format!(
            "Version {}",
            iph.ihl_v.get_version()
        )));
    }

    let iphlen = iph.ihl_v.get_hdrsize();
    if iphlen < 20 {
        return Err(NetErr::AnyWay(format!("IP Header too small {iphlen}")));
    }

    let iplen = skb.total_len - ETH_HLEN as u32;
    if iplen < iph.len.native() as u32 {
        return Err(NetErr::AnyWay(format!("Invalid IP package total len",)));
    }

    if iphlen < iph.len.native() as usize {
        return Err(NetErr::AnyWay(format!(
            "IP header smaller than package len"
        )));
    }

    if cksum(skb.nh.raw, IPHLEN as u16) != 0 {
        skb.ip_checked = true;
    }

    if iph.ip_dst != dev.ip_host
        && (iph.ip_dst.ipv4().is_broadcast()
            || iph.ip_src.ipv4().is_broadcast())
    {
        return Err(NetErr::AnyWay(format!("No local and no broadcast")));
    }

    Ok(())
}


pub unsafe fn ip_input(dev: &NetDevice, mut skb: SKBuff) -> Result<()> {
    validate_ip(dev, &mut skb)?;

    let iph = &*skb.nh.iph;

    if iph.frag_off.get_frag_off_size() > 0 {
        // Reassemable
    }

    match iph.protocol {
        Protocol::ICMP => {
            todo!()
        }
        Protocol::UDP => {}
        _ => todo!(),
    }

    Ok(())
}


#[allow(unused)]
pub unsafe fn ip_output(
    dev: &NetDevice,
    mut skb: SKBuff,
    src: InAddrN,
    dst: InAddrN,
) -> Result<()> {
    let mut iph = &mut *skb.nh.iph;
    iph.ip_dst = dst;
    iph.ip_src = src;
    iph.checksum = 0;
    cksum(skb.nh.raw, size_of::<IP>() as u16);
    skb.curproto_len = skb.total_len;

    if skb.curproto_len > dev.mtu as u32 {
        /* fragmentation */
        skb = ip_frag(dev, skb);
    }

    dev.output(&skb)
}


/// Do IP package fragmentation
unsafe fn ip_frag(dev: &NetDevice, mut skb: SKBuff) -> SKBuff {
    let mtu = dev.mtu as usize;
    // let half_mtu = ((dev.mtu + 1) / 2) as usize;
    let plen = (*skb.nh.iph).len.native() as usize;

    // Eth hdr + IP hdr length
    let ethip_hdr_len = ETH_HLEN + IPHLEN;

    /* create the first slice */
    let mut consumed = mtu >> 3 << 3;
    let mut skb_c: *mut SKBuff = &skb as *const SKBuff as _;

    let mut skb_t = SKBuff::with_capcity(consumed);
    skb_t.phy.raw = skb_t.forward(ETH_HLEN);
    skb_t.nh.raw = skb_t.forward(IPHLEN);
    // copy from head straightly
    memcpy(skb_t.head as _, skb.head as _, consumed);
    skb.forward(consumed);
    (*skb_t.nh.iph).frag_off = FragOff::new(FragFlag::MF, 0);
    (*skb_t.nh.iph).checksum = cksum(skb_t.nh.raw, IPHLEN as u16);

    (*skb_c).next = &mut skb_t as _;
    skb_c = (*skb_c).next;

    /* create the one to the last but one slice */
    let step = (mtu - ethip_hdr_len) >> 3 << 3;

    while consumed + step < plen {
        let mut skb_t = SKBuff::with_capcity(ethip_hdr_len + step);
        skb_t.phy.raw = skb_t.forward(ETH_HLEN);
        skb_t.nh.raw = skb_t.forward(IPHLEN);

        // just copy eth && ip header
        memcpy(skb_t.head as _, skb.head as _, ethip_hdr_len);
        memcpy(skb_t.head.add(ethip_hdr_len) as _, skb.tail as _, step);
        skb.forward(step);

        debug_assert!(consumed % 8 == 0);
        (*skb_t.nh.iph).frag_off =
            FragOff::new(FragFlag::MF, consumed.div_euclid(8) as u16);
        (*skb_t.nh.iph).checksum = cksum(skb_t.nh.raw, IPHLEN as u16);

        (*skb_c).next = &mut skb_t as _;
        skb_c = (*skb_c).next;

        consumed += step;
    }

    /* create the last slice */
    let rem = plen - consumed;
    let mut skb_t = SKBuff::with_capcity(ethip_hdr_len + step);
    skb_t.phy.raw = skb_t.forward(ETH_HLEN);
    skb_t.nh.raw = skb_t.forward(IPHLEN);

    // just copy eth && ip header
    memcpy(skb_t.head as _, skb.head as _, ethip_hdr_len);
    memcpy(skb_t.head.add(ethip_hdr_len) as _, skb.tail as _, rem);
    skb.forward(rem);
    (*skb_t.nh.iph).frag_off =
        FragOff::new(FragFlag::MF, consumed.div_euclid(8) as u16);
    (*skb_t.nh.iph).checksum = cksum(skb_t.nh.raw, IPHLEN as u16);

    (*skb_c).next = &mut skb_t as _;
    skb_c = (*skb_c).next;

    (*skb_c).next = null_mut();

    skb
}


/// Do IP package reassemable
#[allow(unused)]
unsafe fn ip_reass(mut skb: SKBuff) {
    let fragh = &*skb.nh.iph;

    let offset = fragh.frag_off.get_frag_off_size();
    let plen = fragh.len.native() as usize;


    unimplemented!()
}
