#![feature(box_syntax)]
#![allow(unused_imports)]

use std::error::Error;
use std::ffi::CString;
use std::mem::{size_of, transmute, zeroed, MaybeUninit};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::ptr::{null, null_mut, slice_from_raw_parts, self};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, RwLock};
use std::thread::{self, Thread, sleep};
use std::time::{Duration, Instant};

use bincode::{options, Options};
use clap::Parser;
use either::Either;
use libc::{
    __errno_location, c_void, fd_set, getpid, getprotobyname, recv, select,
    sendto, setsockopt, signal, sockaddr, sockaddr_in, socket, timeval,
    AF_INET, EINTR, FD_SET, FD_ZERO, PT_NULL, SIGINT, SOCK_RAW, SOL_SOCKET,
    SO_RCVBUF,
};
use netlib::data::SockAddrIn;
use netlib::network::ip::Protocol;
use netlib::{__item, cstr, defe};
use netlib::{
    aux::From2,
    bincode_options,
    network::{
        icmp::{ICMPType, ICMP},
        ip::IP,
    },
};
use signal_hook::consts;
use signal_hook::flag::register;


////////////////////////////////////////////////////////////////////////////////
//// Erro

defe! {
    pub enum PingError {
        UnHandledICMP(ICMPType, i32),
        UnmatchedPacketSeq(u16),
        SendToFailed,
        CreateRawSocketFailed(i32),
        UnresolvedHost(String),
        MMPoisonError
    }

    pub enum CliError {
        ParseInAddrFailed(String)
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Macros

macro_rules! get_packet_mut {
    ($packets:ident, $seq:expr) => {
        $packets
            .iter_mut()
            .find(|pac| pac.seq == $seq)
            .ok_or(box PingError::UnmatchedPacketSeq($seq))
    };
}

#[macro_export(inner_macros)]
macro_rules! push_packet {
    ($packets:expr) => {{
        let mut packets = $packets;
        let seq = packets.len();
        packets.push(PingPacket::new(seq as u16));
    }};
}


////////////////////////////////////////////////////////////////////////////////
//// Structs

#[derive(Debug)]
pub struct PingPacket {
    sent_time: Instant,
    seq: u16,
    received: bool,
}



////////////////////////////////////////////////////////////////////////////////
//// Implements

fn icmp_pack(buf: &mut [u8], seq: u16, payload_size: u8) {
    let config = bincode_options!();

    let ty: u8 = ICMPType::EchoRequest.into();
    let code = 0;
    let cksum = 0;
    let pid = unsafe { getpid() };
    let un = ICMP::un_as_echo((pid & 0xffff) as u16, seq);

    let mut icmp = ICMP {
        ty,
        code,
        cksum,
        un,
    };

    config
        .serialize_into(&mut buf[..size_of::<ICMP>()], &icmp)
        .unwrap();
    for i in 0..payload_size {
        buf[size_of::<ICMP>() + i as usize] = i as u8; // create non-zero data to check sum
    }

    icmp.cksum = unsafe {
        ICMP::calc_cksum(
            buf.as_mut_ptr(),
            payload_size as u32 + size_of::<ICMP>() as u32,
        )
    };

    config
        .serialize_into(&mut buf[..size_of::<ICMP>()], &icmp)
        .unwrap();
}


unsafe fn icmp_send(
    rawsock: i32,
    dst: sockaddr_in,
    packets: Arc<Mutex<Vec<PingPacket>>>,
    init_signal_arrived: Arc<AtomicBool>,
) -> Result<(), PingError> {
    let payload_size = 64;
    let mut sendbuf = [0u8; 64 + size_of::<ICMP>()];
    let ipv4_dst = Ipv4Addr::from(dst.sin_addr.s_addr);

    let mut packets_sent: usize = 0;
    while !init_signal_arrived.load(Ordering::Relaxed) {
        let seq = packets_sent as u16;
        icmp_pack(&mut sendbuf, seq, payload_size);

        let size = sendto(
            rawsock,
            sendbuf.as_ptr() as *const c_void,
            sendbuf.len(),
            0,
            &dst as *const sockaddr_in as *const sockaddr,
            size_of::<sockaddr_in>() as u32,
        );

        if size < 0 {
            eprintln!(
                "sendto {:#?} failed",
                ipv4_dst
            );
            break;
        }
        else {
            println!("send to {:#?} {} bytes", ipv4_dst, size)
        }

        push_packet!(packets.lock().unwrap());
        packets_sent += 1;

        sleep(Duration::new(1, 0));
    }

    Ok(())
}


unsafe fn icmp_unpack(
    buf: &mut [u8],
    packets: Arc<Mutex<Vec<PingPacket>>>,
) -> Result<(), Box<dyn Error>> {
    let config = bincode_options!().allow_trailing_bytes();
    println!("icmp unpack...");

    let iphdr: IP = config.deserialize(&buf[..])?;
    // let iphdr: IP = ptr::read (buf.as_ptr() as *const _);
    println!("icmp unpack... -> unpack iphdr");

    let iphdr_len = iphdr.get_hdrsize();
    println!("icmp unpack... -> get iphdr len ({})", iphdr_len);

    let icmphdr: ICMP = config.deserialize(&buf[iphdr_len..])?;

    let (id, seq) = icmphdr.get_idseq();
    let pid = getpid();

    println!("reply type: {:#?}", icmphdr.parse_cm_type());

    if icmphdr.parse_cm_type()? == ICMPType::EchoReply
        && id == (pid & 0xffff) as u16
    {
        match packets.lock() {
            Ok(mut packets_guard) => {
                let packet = get_packet_mut!(packets_guard, seq)?;

                packet.received = true;

                let src = iphdr.get_src_ip();
                let rrt = Instant::now().duration_since(packet.sent_time);

                println!(
                    "{} bytes from {:#?}: icmp_seq={} ttl={} rtt={:?}",
                    buf.len(),
                    src,
                    seq,
                    iphdr.ttl(),
                    rrt,
                );
            }
            // 这个结构设计得，不能直接返回，真的不太行
            Err(_err) => unreachable!(),
        }
    } else {
        return Err(box PingError::UnHandledICMP(ICMPType::EchoReply, pid));
    }

    Ok(())
}


unsafe fn icmp_recv(
    rawsock: i32,
    packets: Arc<Mutex<Vec<PingPacket>>>,
    init_signal_arrived: Arc<AtomicBool>,
) -> Result<(), PingError> {
    let mut recv_buf = [0u8; 2 * 1024];
    let mut readfd: fd_set = zeroed(); // bits map

    while !init_signal_arrived.load(Ordering::Relaxed) {
        // select return will clear all bit but the ready bit
        // FD_ZERO(&mut readfd);
        FD_SET(rawsock, &mut readfd);
        // select modifed the timeval (pselect copy)
        let mut timeout: timeval = timeval {
            tv_sec: 2,
            tv_usec: 0,
        };  // set 200ms timeout
        let ret = select(
            rawsock + 1,
            &mut readfd,
            null_mut(),
            null_mut(),
            &mut timeout,
        );

        // 0 timeout, -1 errors

        if ret == -1 {
            eprintln!("select error");
            continue;
        }

        if ret == 0 {
            eprintln!("select timeout");
            continue;
        }
        println!("select notified");

        let size = recv(
            rawsock,
            recv_buf.as_mut_ptr() as *mut c_void,
            recv_buf.len(),
            0,
        );

        if *__errno_location() == EINTR {
            eprintln!("recvfrom error");
            continue;
        }

        debug_assert!(size > 0);
        // println!("recvd -> unpack");
        if icmp_unpack(&mut recv_buf[..size as usize], packets.clone()).is_ok()
        {
            break;
        }
    }

    Ok(())
}


fn statistics(dst: Ipv4Addr, packets: Arc<Mutex<Vec<PingPacket>>>) {
    println!("--- {:?} ping statistics ---", dst);

    let packets_guard = packets.lock().unwrap();

    let mut packets_sent = 0;
    let mut packets_received = 0;

    let pingstart = packets_guard[0].sent_time;
    let pingend = Instant::now();
    let time = pingend - pingstart;

    for packet in packets_guard.iter() {
        if packet.received {
            packets_received += 1;
        } else {
            packets_sent += 1;
        }
    }

    println!(
        "{} packets transmitted, {} received, {}% packet loss, time: {} ms",
        packets_sent,
        packets_received,
        (packets_sent - packets_received) * 100 / packets_sent,
        time.as_millis()
    )
}


impl PingPacket {
    fn new(seq: u16) -> Self {
        Self {
            sent_time: Instant::now(),
            seq,
            received: false,
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Cli

struct HostOrIPv4(Either<Ipv4Addr, String>);

// union HostOrIPv4 {
//     ip: Ipv4Addr,
//     hostname: std::mem::ManuallyDrop<String>
// }


impl FromStr for HostOrIPv4 {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(c) = s.chars().next() {
            Ok(if c.is_digit(10) {
                HostOrIPv4(Either::Left(Ipv4Addr::from_str(s)?))
            } else {
                HostOrIPv4(Either::Right(s.to_owned()))
            })
        } else {
            Err(box CliError::ParseInAddrFailed("".to_string()))
        }
    }
}

impl TryInto<Ipv4Addr> for HostOrIPv4 {
    type Error = Box<dyn Error>;

    fn try_into(self) -> Result<Ipv4Addr, Self::Error> {
        Ok(match self.0 {
            Either::Left(ip) => ip,
            Either::Right(hostname) => {
                let addrs =
                    dns_lookup::getaddrinfo(Some(&hostname), None, None)
                        .or_else(|_lkerr| {
                            Err(box PingError::UnresolvedHost(hostname))
                        })?
                        .collect::<std::io::Result<Vec<_>>>()
                        .unwrap();

                let addr1st = &addrs[0];

                match addr1st.sockaddr.ip() {
                    std::net::IpAddr::V4(ip) => ip,
                    std::net::IpAddr::V6(_) => unreachable!(),
                }
            }
        })
    }
}



#[derive(Parser)]
#[clap()]
struct Cli {
    #[clap()]
    dst: String,
}



fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let dst = cli.dst;

    // println!("ping to {:?}", dst);

    unsafe {
        let hostorip = HostOrIPv4::from_str(&dst)?;

        let dst: Ipv4Addr = hostorip.try_into()?;
        let cdst = transmute::<SockAddrIn, sockaddr_in>(SockAddrIn::from(dst));

        let rawsock = socket(AF_INET, SOCK_RAW, Protocol::ICMP as i32);

        if rawsock < 0 {
            return Err(box PingError::CreateRawSocketFailed(rawsock));
        }

        let size = 128 * 1024;
        setsockopt(
            rawsock,
            SOL_SOCKET,
            SO_RCVBUF,
            &size as *const i32 as *const c_void,
            size_of::<i32>() as u32,
        );

        let init_signal_arrived = Arc::new(AtomicBool::new(false));
        register(consts::SIGINT, init_signal_arrived.clone())?;

        println!("PING ({:?}) 56(84) bytes of data.", dst);

        let packets = Arc::new(Mutex::new(Vec::new()));

        let packets_send = packets.clone();
        let init_signal_arrived_send = init_signal_arrived.clone();

        let thd_send = thread::Builder::new()
            .name("child-send".to_owned())
            .spawn(move || {
                icmp_send(
                    rawsock,
                    cdst,
                    packets_send,
                    init_signal_arrived_send,
                )
            })?;

        let packets_recv = packets.clone();
        let init_signal_arrived_recv = init_signal_arrived.clone();

        let thd_recv = thread::Builder::new()
            .name("child-recv".to_owned())
            .spawn(move || {
                icmp_recv(rawsock, packets_recv, init_signal_arrived_recv)
            })?;

        thd_send.join().unwrap()?;
        thd_recv.join().unwrap()?;

        statistics(dst, packets)
    }


    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::CString,
        fmt::Debug,
        mem::{zeroed, MaybeUninit},
        ptr::{null, null_mut},
    };

    use dns_lookup::getaddrinfo;
    use libc::{addrinfo, fd_set, gai_strerror, FD_ISSET, FD_SET, FD_ZERO};
    use netlib::cstr;


    #[test]
    fn test_fdset() {
        unsafe {
            // bits map
            let mut readfd: fd_set = MaybeUninit::uninit().assume_init();
            let rawsock = 2;

            // FD_ZERO(&mut readfd);
            FD_SET(rawsock, &mut readfd);

            println!("1 is_set {:?}", FD_ISSET(0, &readfd));
            println!("2 is_set {:?}", FD_ISSET(rawsock, &readfd));
        }
    }

    #[test]
    fn test_getaddrinfo() {
        // unsafe {
        //     // let mut res: *mut addrinfo = null_mut();

        //     // let s = getaddrinfo(cstr!("baidu2.com"), null(), null(), &mut res);
        //     // if s != 0 {
        //     //     eprintln!("getaddrinfo: {:?}", gai_strerror(s))
        //     // }
        //     // else {
        //     //     println!("getaddrinfo: {:?}", (*(*res).ai_addr) );
        //     // }

        // }
        let hostname = "baidu.com";

        let sockets = getaddrinfo(Some(hostname), None, None)
            .unwrap()
            .collect::<std::io::Result<Vec<_>>>()
            .unwrap();

        for socket in sockets {
            println!("{:?}", socket);
        }
    }

    #[test]
    fn test_shf() {
        let n = 0u32;

        let n2 = n >> 16 + n;

        println!("{}", n2);

        let n3: u32 = 0xffff_ffff >> 16;
        println!("{}", n3);

        let n4: u8 = 123;
        println!("{}", ((n4 as u16) << 8) as u16 & 0xff00)

    }

}
