#![feature(box_syntax)]
#![feature(local_key_cell_methods)]

use std::{
    error::Error,
    mem::{size_of, zeroed},
    net::Ipv4Addr,
    ptr::{null_mut, read, write},
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, sleep},
    time::{Duration, Instant},
};

use clap::Parser;
use libc::{
    __errno_location, c_void, fd_set, getpid, recv, select, sendto,
    setsockopt, sockaddr, sockaddr_in, socket, timeval, AF_INET, EINTR,
    FD_SET, FD_ZERO, IPPROTO_IP, IP_MULTICAST_IF, IP_MULTICAST_TTL, IP_TTL,
    SOCK_RAW, SOL_SOCKET, SO_BROADCAST, SO_RCVBUF, signal, SIGINT, memset,
};
use netlib::{
    aux::HostOrIPv4,
    data::SockAddrIn,
    defe,
    network::{
        icmp::{ICMPType, ICMP},
        inet_cksum,
        ip::{Protocol, IP},
    }, size,
};


const ICMP_PAYLOAD_SIZE: usize = 0;
const ICMP_PACK_SIZE: usize = size!(ICMP) + ICMP_PAYLOAD_SIZE;

lazy_static::lazy_static! {
    static ref SELECT_TIMEOUT: Arc<timeval> = {
        Arc::new(
            timeval { tv_sec: 0, tv_usec: 0 }
        )
    };
    static ref PING_PACKS: Arc<Mutex<Vec<PingPacket>>> = {
        Arc::new(Mutex::new(Vec::new()))
    };

    static ref HAS_SIG_INI: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

}

static mut SELECT_TIMEOUT_P: Option<usize> = None;


////////////////////////////////////////////////////////////////////////////////
//// Error

defe! {
    pub enum PingError {
        UnHandledICMP(ICMPType, i32),
        UnmatchedPacketSeq(u16),
        SendToFailed,
        CreateRawSocketFailed(i32),
        UnresolvedHost(String),
        MMPoisonError,
        DeserializeFailed(String)
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Macros

macro_rules! get_packet_mut {
    ($packets:ident, $seq:expr) => {
        $packets
            .iter_mut()
            .find(|pac| pac.seq == $seq)
            .ok_or(PingError::UnmatchedPacketSeq($seq))
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

#[macro_export(inner_macros)]
macro_rules! init_select_timeout {
    () => {
        {
            let p = Arc::as_ptr(&SELECT_TIMEOUT) as *mut timeval;

            // let p = SELECT_TIMEOUT.as_();

            (*p).tv_sec = 10;
            (*p).tv_usec = 0;

            SELECT_TIMEOUT_P.replace(p as usize);
            p
        }

        // (timeval {
        //     tv_sec: 2,
        //     tv_usec: 0
        // })
    };
}



////////////////////////////////////////////////////////////////////////////////
//// Structs

#[derive(Debug)]
pub struct PingPacket {
    sent_time: Instant,
    seq: u16,
    received: bool,
}

enum UnpackRes {
    Succcess,
    Retry,
}


////////////////////////////////////////////////////////////////////////////////
//// Implements

unsafe fn icmp_pack(buf: &mut [u8], seq: u16) {
    let ty: u8 = ICMPType::EchoRequest.into();
    let code = 0;
    let cksum = 0;
    let pid = getpid();
    let un = ICMP::un_as_echo((pid & 0xffff) as u16, seq);

    let mut icmp = ICMP {
        ty,
        code,
        cksum,
        un,
    };

    // println!(
    //     "pack icmp id: {}, seq: {:0x}",
    //     icmp.get_idseq().0,
    //     icmp.get_idseq().1
    // );

    write(buf[..size_of::<ICMP>()].as_mut_ptr() as *mut ICMP, icmp);

    // for i in 0..(icmp_packet_len as usize - size_of::<ICMP>()) {
    //     buf[size_of::<ICMP>() + i as usize] = i as u8; // create non-zero data to check sum
    // }
    write(
        buf[size_of::<ICMP>()..ICMP_PACK_SIZE].as_mut_ptr(),
        zeroed(),
    );

    icmp.cksum = inet_cksum(buf.as_mut_ptr(), ICMP_PACK_SIZE);
    write(buf[..size_of::<ICMP>()].as_mut_ptr() as *mut ICMP, icmp);

}


unsafe fn icmp_unpack(
    buf: &mut [u8],
    packets: Arc<Mutex<Vec<PingPacket>>>,
) -> Result<UnpackRes, PingError> {
    let iphdr: IP = read(buf.as_ptr() as *const _);

    // println!("fragflag: {:?}, fragoff: {}, raw: {}", iphdr.get_frag_flag(), iphdr.get_frag_off(), iphdr.frag_off);

    let icmphdr: ICMP = read(buf[size_of::<IP>()..].as_ptr() as *const _);

    let (id, seq) = icmphdr.get_idseq();
    let pid = getpid();

    // println!("reply type: {:#?}", icmphdr.parse_cm_type());
    // println!("reply seq: {:0x}, id: {:0x}, pid: {:0x}", seq, id, pid);
    // println!("reply dst: {}", Ipv4Addr::from(ntohl(iphdr.ip_dst)));

    let icmp_type = icmphdr.parse_cm_type().or_else(|err| {
        Err(PingError::DeserializeFailed(format!("icmptype: {err:?}")))
    })?;

    if icmp_type != ICMPType::EchoReply {
        eprintln!("{icmp_type:?}");
        return Ok(UnpackRes::Retry);
    }
    if id != (pid & 0xffff) as u16 {
        eprintln!("Expected id {}, found {}", id, (pid & 0xffff) as u16);
        return Ok(UnpackRes::Retry);
    }

    match packets.lock() {
        Ok(mut packets_guard) => {
            let packet = get_packet_mut!(packets_guard, seq)?;

            packet.received = true;

            let src = iphdr.get_src_ip();
            let rrt = Instant::now().duration_since(packet.sent_time);

            let rrt_micros = rrt.as_micros();

            println!(
                "{}({}) bytes from {:#?}: icmp_seq={} ttl={} rtt={:.3} ms",
                iphdr.len.native() as usize - size!(IP),
                iphdr.len.native(),
                src,
                seq,
                iphdr.ttl,
                (rrt_micros as f64) / (10u32.pow(3) as f64),
            );
        }
        // 这个结构设计得，不能直接返回，真的不太行
        Err(_err) => unreachable!(),
    }
    // else just return

    Ok(UnpackRes::Succcess)
}


unsafe fn ping_once(
    rawsock: i32,
    sendbuf: &mut [u8],
    packets: Arc<Mutex<Vec<PingPacket>>>,
    dst: sockaddr_in,
) -> Result<(), PingError> {
    let ipv4_dst = Ipv4Addr::from(dst.sin_addr.s_addr);

    let seq = packets.lock().unwrap().len() as u16;
    icmp_pack(sendbuf, seq);

    let size = sendto(
        rawsock,
        sendbuf.as_ptr() as *const c_void,
        sendbuf.len(),
        0,
        &dst as *const sockaddr_in as *const sockaddr,
        size_of::<sockaddr_in>() as u32,
    );

    if size < 0 {
        eprintln!("sendto {:#?} failed", ipv4_dst);
        return Err(PingError::SendToFailed);
    }
    else {
        // println!("send to {:#?} {} bytes", ipv4_dst, size)
    }

    push_packet!(packets.lock().unwrap());

    Ok(())
}


unsafe fn interrupt_select() {
    let p = SELECT_TIMEOUT_P.unwrap() as *mut timeval;
    memset(p as _, 0, size!(timeval));

    let inip = Arc::as_ptr(&HAS_SIG_INI) as *mut AtomicBool;
    *inip = AtomicBool::new(true);
}


unsafe fn ping_recv_loop(
    rawsock: i32,
    packets: Arc<Mutex<Vec<PingPacket>>>,
    dst: sockaddr_in,
    init_signal_arrived: Arc<AtomicBool>,
) -> Result<(), PingError> {
    let mut recv_buf = [0u8; 2 * 1024];
    let mut readfd: fd_set = zeroed(); // bits map
    let mut send_buf = [0u8; 68]; // 56 + 8 + 4

    loop {
        ping_once(rawsock, &mut send_buf, packets.clone(), dst)?;

        // select return will clear all bit but the ready bit
        FD_ZERO(&mut readfd);
        FD_SET(rawsock, &mut readfd);
        // select modifed the timeval (pselect copy)
        // let mut timeout: timeval = timeval {
        //     tv_sec: 2,
        //     tv_usec: 0,
        // }; // set 200ms timeout

        let timeout = init_select_timeout!();
        let ret = select(
            rawsock + 1,
            &mut readfd,
            null_mut(),
            null_mut(),
            timeout,
        );

        if init_signal_arrived.load(Ordering::Relaxed) {
            break;
        }

        // -1 errors
        if ret == -1 {
            eprintln!("select error");
            continue;
        }

        // 0 timeout
        if ret == 0 {
            eprintln!("timeout");
            continue;
        }

        let size = recv(
            rawsock,
            recv_buf.as_mut_ptr() as *mut c_void,
            recv_buf.len(),
            0,
            // &mut dst as *mut sockaddr_in as *mut sockaddr,
            // &mut size_of::<sockaddr_in>() as *mut usize as *mut u32
        );

        if *__errno_location() == EINTR {
            eprintln!("recvfrom error");
            continue;
        }

        debug_assert!(size > 0);
        match icmp_unpack(&mut recv_buf[..size as usize], packets.clone())? {
            UnpackRes::Succcess => {}
            UnpackRes::Retry => {
                // eprintln!("retry...");
            }
        }

        sleep(Duration::new(0, 500_000_000));
    }

    Ok(())
}


fn statistics(dst: Ipv4Addr, packets: Arc<Mutex<Vec<PingPacket>>>) {
    println!("--- {:?} ping statistics ---", dst);

    let packets_guard = packets.lock().unwrap();

    let packets_sent = packets_guard.len();
    let mut packets_received = 0;

    let pingstart = packets_guard[0].sent_time;
    let pingend = Instant::now();
    let time = pingend - pingstart;

    for packet in packets_guard.iter() {
        if packet.received {
            packets_received += 1;
        }
    }

    println!(
        "{} packets transmitted, {} received, {}% packet loss, time: {:.3} ms",
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
        let mut cdst = SockAddrIn::from(dst).into();

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

        // enable the outcoming interface multicasting
        setsockopt(
            rawsock,
            IPPROTO_IP,
            IP_MULTICAST_IF,
            &mut cdst as *mut sockaddr_in as *mut c_void,
            size_of::<sockaddr_in>() as u32,
        );

        // enable broadcast pings
        setsockopt(
            rawsock,
            SOL_SOCKET,
            SO_BROADCAST,
            &size as *const i32 as *const c_void,
            size_of::<i32>() as u32,
        );

        // seconds
        let ttl = 200u8;
        // set TTL
        setsockopt(
            rawsock,
            IPPROTO_IP,
            IP_TTL,
            &ttl as *const u8 as *const c_void,
            size_of::<u8>() as u32,
        );
        setsockopt(
            rawsock,
            IPPROTO_IP,
            IP_MULTICAST_TTL,
            &ttl as *const u8 as *const c_void,
            size_of::<u8>() as u32,
        );

        println!(
            "PING ({:?}) {}({}) bytes of data.",
            dst, ICMP_PACK_SIZE, ICMP_PACK_SIZE + size!(IP)
        );

        // use signal_hook::{consts, flag::register};
        // let init_signal_arrived = Arc::new(AtomicBool::new(false));
        // register(consts::SIGINT, init_signal_arrived.clone())?;
        // let init_signal_arrived_recv = init_signal_arrived.clone();

        signal(SIGINT, interrupt_select as unsafe fn() as usize);
        let packets_recv = PING_PACKS.clone();
        let init_signal_arrived_recv = HAS_SIG_INI.clone();

        let thd_recv = thread::Builder::new()
            .name("child-recv".to_owned())
            .spawn(move || {
                ping_recv_loop(
                    rawsock,
                    packets_recv,
                    cdst,
                    init_signal_arrived_recv,
                )
            })?;

        thd_recv.join().unwrap()?;

        statistics(dst, PING_PACKS.clone())
    }


    Ok(())
}

#[cfg(test)]
mod tests {}
