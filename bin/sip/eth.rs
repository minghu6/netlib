use std::{
    fmt::Debug,
    mem::{size_of, transmute, zeroed},
    net::IpAddr,
};

use libc::{
    bind, memcpy, read, sendto, sleep, sockaddr, socket, socklen_t, strcpy,
    AF_INET, ETH_FRAME_LEN, IFNAMSIZ, SOCK_PACKET,
};
use log::{ info, debug };
use netlib::{
    data::{getgateway, getifaddrs, FixStr, InAddrN, SAFamily, Subnet},
    datalink::{Eth, EthTypeE, EthTypeN, Mac},
    defraw1,
    error::{NetErr, Result},
    network::{arp::ARP, ip::IP},
    or2anyway, throw_errno,
};

use crate::{
    arp::{arp_input, arp_req, ARPLIVE, ARPTAB},
    ip::ip_input,
    skbuff::SKBuff,
};


pub const ETH_HLEN: usize = size_of::<Eth>();


////////////////////////////////////////////////////////////////////////////////
//// Structure

defraw1! {
    /// Mock a real net device
    pub struct NetDevice {
        name: FixStr<IFNAMSIZ>,
        ip_host: InAddrN,
        ip_netmask: InAddrN,
        ip_broadcast: InAddrN,
        ip_gateway: InAddrN,
        ip_dst: InAddrN,
        type_: EthTypeN,

        /// 从网络设备中获取数据，传入协议栈进行处理
        input: *mut unsafe fn(&NetDevice) -> Result<()>,
        /// IP模块发送数据时调用，经过ARP模块
        output: *mut unsafe fn(&NetDevice, &SKBuff) -> Result<()>,
        /// ARP模块调用
        linkoutput: *mut unsafe fn(&NetDevice, &SKBuff,) -> Result<()>,

        hwa_len: u8,
        hwa: Mac,
        hwa_broadcast: Mac,
        mtu: u16,
        /// Sock descriptor
        sd: i32,
        to: sockaddr
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl NetDevice {
    pub unsafe fn init(ifname: &str) -> Result<Self> {
        let mut dev: Self = zeroed();

        dev.sd = throw_errno!(socket(
            AF_INET,
            SOCK_PACKET,
            EthTypeE::PAll.net().val() as i32
        ) throws CreateRawSocket);

        dev.name = ifname.parse().unwrap();
        dev.to.sa_family = transmute(SAFamily::Inet);
        strcpy(dev.to.sa_data.as_mut_ptr(), dev.name.as_ptr() as *const _);

        throw_errno!(
            bind(dev.sd, &dev.to, size_of::<sockaddr>() as u32)
            throws Bind
        );

        dev.hwa_broadcast = Mac::broadcast();
        // dev.hwa = Mac::new(0x00, 0x0c, 0x29, 0x73, 0x9d, 0x1f);
        dev.hwa = or2anyway!("00:12:34:56:78:90".parse())?;
        dev.hwa_len = size_of::<Mac>() as u8;

        /* bind if info */
        let ifaddrs = getifaddrs()?;
        if let Some((_name, ip, mask)) = ifaddrs
            .get_inet_items()
            .find(|(name, _, _)| *name == ifname)
        {
            dev.ip_host = InAddrN::from_ipv4addr(*ip);
            dev.ip_netmask = InAddrN::from_ipv4addr(*ip);
            dev.ip_broadcast = InAddrN::from_ipv4addr(ip.broadcast(mask));
        }
        else {
            return Err(NetErr::AnyWay(format!("No such if {ifname}")));
        }

        /* bind gateway */
        let gateway = getgateway()?;
        let ip_gateway = match gateway.ip_addr {
            IpAddr::V4(ipv4) => ipv4,
            IpAddr::V6(ipv6) => {
                return Err(NetErr::AnyWay(format!("{ipv6:?}")))
            }
        };
        dev.ip_gateway = InAddrN::from_ipv4addr(ip_gateway);

        dev.input = input as *mut _;
        dev.output = output as *mut _;
        dev.linkoutput = linkoutput as *mut _;

        dev.mtu = ETH_FRAME_LEN as u16;
        dev.type_ = EthTypeE::P8023.net();

        Ok(dev)
    }
}


impl Debug for NetDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetDevice")
            .field("name", &self.name)
            .field("ip_host", &self.ip_host)
            .field("ip_netmask", &self.ip_netmask)
            .field("ip_broadcast", &self.ip_broadcast)
            .field("ip_gateway", &self.ip_gateway)
            .field("ip_dst", &self.ip_dst)
            .field("type_", &self.type_)
            // .field("input", &self.input)
            // .field("output", &self.output)
            // .field("linkoutput", &self.linkoutput)
            .field("hwa_len", &self.hwa_len)
            .field("hwa", &self.hwa)
            .field("hwa_broadcast", &self.hwa_broadcast)
            .field("mtu", &self.mtu)
            .field("sd", &self.sd)
            .field("to", &self.to)
            .finish()
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Function

/// 从网卡输入数据

pub unsafe fn input(dev: &NetDevice) -> Result<()> {
    info!("dev input ...");
    let mut ef: [u8; ETH_FRAME_LEN as usize] = zeroed();

    let n = throw_errno!(
        read(dev.sd, ef.as_mut_ptr() as *mut _, ETH_FRAME_LEN as usize)
        throws AnyWay withs
    ) as usize;

    let mut skb = SKBuff::with_capcity(n);
    memcpy(skb.head as *mut _, ef.as_ptr() as *const _, n);

    skb.phy.ethh = skb.forward(size_of::<Eth>()) as *mut _;
    let ethh = &*skb.phy.ethh;

    if [dev.hwa, dev.hwa_broadcast]
        .into_iter()
        .any(|x| x == ethh.dst)
    {
        match ethh.proto.native()? {
            EthTypeE::IPv4 => {
                skb.nh.iph = skb.forward(size_of::<IP>()) as *mut _;

                ARPTAB.with_borrow_mut(|tab| {
                    tab.insert((*skb.nh.iph).ip_src, ethh.src, ARPLIVE as i64)
                });
                debug!("Incomming Network IPv4 handled {:?}", (*skb.nh.iph).ip_dst);
                ip_input(dev, skb)?;

            }
            EthTypeE::ARP => {
                skb.nh.arph = skb.forward(size_of::<ARP>()) as *mut _;
                let arph = &*skb.nh.arph;
                let arptip = arph.tip;

                if arptip == dev.ip_host {
                    debug!("Incomming Network ARP handled {:?}", arptip);
                    arp_input(dev, skb)?;
                }
                else {
                    debug!("Incomming Network filtered {:?}", arptip);
                }
            }
            _ => {}
        }
    }
    else {
        debug!("Incomming Link filtered {:?}", ethh.dst);
    }

    Ok(())
}


/// 底层发送
pub unsafe fn linkoutput(skbuff: &SKBuff, dev: &NetDevice) -> Result<()> {
    let mut p = skbuff as *const SKBuff;

    while !p.is_null() {
        let skp = &*p;

        let n = throw_errno!(
            sendto(dev.sd, skp.head as *mut _, skp.curproto_len as usize, 0, &dev.to, size_of::<sockaddr> as socklen_t)
            throws SendTo
        );
        info!("send {n} bytes");

        p = (*p).next
    }

    Ok(())
}


/// 从网卡输出数据
pub unsafe fn output(skbuff: &SKBuff, dev: &NetDevice) -> Result<()> {
    let mut dst_ip = (*skbuff.nh.iph).ip_dst;

    // is same subnet
    if dst_ip.subnet(&dev.ip_netmask) == dev.ip_host.subnet(&dev.ip_netmask) {
        /* Send to gateway */
        dst_ip = dev.ip_gateway;
    }

    let mut rec_opt = None;
    for _ in 0..5 {
        rec_opt = ARPTAB.with_borrow_mut(|tab| {
            tab.get_mut_and_update(dst_ip, ARPLIVE as i64).copied()
        });

        arp_req(dev, dst_ip)?;
        sleep(1);

        if rec_opt.is_some() {
            break;
        }
    }

    if rec_opt.is_none() {
        return Err(NetErr::AnyWay(format!(
            "No Mac found for {:?}",
            dst_ip.ipv4()
        )));
    }

    let dst_mac = rec_opt.unwrap().mac;
    let ethh = &mut (*skbuff.phy.ethh);

    ethh.dst = dst_mac;
    ethh.src = dev.hwa;
    ethh.proto = EthTypeE::IPv4.net();

    if dev.linkoutput.is_null() {
        return Err(NetErr::AnyWay(format!("device linkoutput is null")));
    }

    (*dev.linkoutput)(dev, skbuff)?;

    Ok(())
}
