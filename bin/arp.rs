#![feature(box_syntax)]
#![feature(never_type)]

use std::{mem::{size_of, zeroed}, net::Ipv4Addr, ptr::{write, read, null_mut}, str::FromStr};

use clap::Parser;
use libc::{
    c_void, epoll_create1, epoll_ctl, epoll_event, epoll_wait, sendto,
    sockaddr, socket, AF_PACKET, EPOLLIN, EPOLLET, EPOLLOUT, EPOLL_CTL_ADD, SOCK_RAW,
    recvfrom
};
use m6coll::Array;
use netlib::{
    aux::HostOrIPv4,
    data::{InAddrN, SockAddrLL},
    datalink::{Eth, EthTypeE, Mac, PacType},
    error::{NetErr, Result},
    network::{
        arp::{ARPOpE, ARP, ARPHTE},
        if_::{getifaddrs, getifmac, getifnth},
    },
    or2anyway, throw_errno,
};



const BUF_SIZE: usize = 60;



unsafe fn send_arp(
    sock: i32,
    ifnth: i32,
    src_mac: Mac,
    src_ip: InAddrN,
    dst_ip: InAddrN,
) -> Result<isize> {
    let sockaddr = SockAddrLL {
        family: AF_PACKET as u16,
        proto: EthTypeE::ARP.net(),
        ifindex: ifnth,
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

    write(buf.as_mut_ptr() as *mut Eth, eth);
    write(buf[size_of::<Eth>()..].as_mut_ptr() as *mut ARP, arp);


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


unsafe fn recv_arp(sock: i32,) -> Result<()> {
    let mut buf = [0u8; BUF_SIZE];
    let len = throw_errno!(
        recvfrom(
            sock,
            buf.as_mut_ptr() as *mut _,
            BUF_SIZE,
            0,
            null_mut(),
            null_mut()
        ) throws RecvFrom
    ) as usize;

    let lower = size_of::<Eth>() + size_of::<ARP>();
    if len < lower {
        return Err(NetErr::AnyWay(format!("too small {len}, expect {lower}")));
    }

    let _ethhdr = read(buf.as_mut_ptr() as *mut Eth);
    let arphdr = read(buf[size_of::<Eth>()..].as_mut_ptr() as *mut ARP);

    println!(
        "recv from {} {} to {}",
        arphdr.sip.ipv4(), arphdr.sha,
        arphdr.tip.ipv4()
    );

    Ok(())
}


#[allow(non_camel_case_types)]
#[allow(unused)]
struct epollenv {
    epfd: i32,
    ev: epoll_event,
    events: Array<epoll_event>
}

unsafe fn setup_ev(sock: i32) -> Result<epollenv> {
    let epfd = throw_errno!(epoll_create1(0) throws EpollCreate);

    let ev = epoll_event {
        events: (EPOLLET | EPOLLIN | EPOLLOUT ) as u32,
        u64: sock as u64, // union
    };

    let events = Array::new_with(zeroed(), 100);

    throw_errno!(epoll_ctl(
        epfd,
        EPOLL_CTL_ADD,
        sock,
        &ev as *const epoll_event as *mut epoll_event
    ) throws AnyWay withs );

    Ok(epollenv { epfd, ev, events })
}


unsafe fn ewait_arp(epfd: i32, events: &mut [epoll_event]) -> Result<i32> {
    // return the number of file descriptors ready for the requested I/O
    let nfds = throw_errno!(epoll_wait(
        epfd,
        events.as_mut_ptr(),
        events.len() as i32,
        -1
    ) throws EpollWait);


    Ok(nfds)
}



#[derive(Parser)]
#[clap()]
struct Cli {
    /// Hostname or IP
    #[clap()]
    dst: String,

    /// Set interface by name or else use first nonloop interface
    #[clap(short = 'i')]
    ifname: Option<String>,
}
fn main() -> Result<!> {
    let cli = Cli::parse();

    let hostorip = or2anyway!(HostOrIPv4::from_str(&cli.dst))?;
    let dst: Ipv4Addr = or2anyway!(hostorip.try_into())?;
    let dst_ip = InAddrN::from_ipv4addr(dst);

    unsafe {
        let ifaddrs = getifaddrs()?;
        let mut ifiter = ifaddrs.get_inet_items();
        let (ifname, ip) = if let Some(ifname) = cli.ifname {
            loop {
                if let Some((name, ip, _mask)) = ifiter.next() {
                    if name == ifname {
                        break (name, ip);
                    }
                }
                else {
                    return Err(NetErr::GetIf(format!(
                        "No matched net if name: {ifname}"
                    )));
                }
            }
        }
        else {
            let mut loopback = None;
            loop {
                if let Some((name, ip, _mask)) = ifiter.next() {
                    if !ip.is_loopback() {
                        break (name, ip);
                    }
                    else {
                        loopback = Some((name, ip));
                    }
                }
                else {
                    eprintln!("No nonloopback if detected, using loopback");
                    break loopback.unwrap();
                }
            }
        };

        println!("Using IF ({ifname}: {ip:?})");
        let src_ip = InAddrN::from_ipv4addr(*ip);

        let ifnth = getifnth(ifname).unwrap();
        let src_mac = getifmac(ifname).unwrap();

        let sock =
            socket(AF_PACKET, SOCK_RAW, EthTypeE::ARP.net().val() as i32);
        println!("Send ARP request to {dst_ip:?}");
        send_arp(sock, ifnth, src_mac, src_ip, dst_ip)?;

        let epollenv { epfd, ev: _, mut events } = setup_ev(sock)?;

        let mut i = 0;
        loop {
            let nfds = ewait_arp(epfd, &mut events)?;
            println!("{i} RECV {nfds} Reply");
            recv_arp(sock)?;
            i += 1;
        }
    }


}
