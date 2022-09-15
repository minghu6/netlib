//! On IPv4
#![allow(unused_imports)]
#![allow(unused)]


use std::{mem::size_of, ptr::null_mut, thread};

use libc::{
    accept, bind, epoll_create, epoll_create1, epoll_ctl, epoll_event,
    epoll_wait, sockaddr, sockaddr_in, socket, socketpair, AF_INET, EPOLLIN,
    EPOLL_CTL_ADD, SOCK_RAW, in_addr, INADDR_ANY, socklen_t,
};

use crate::{
    aux::{gen_counter, CounterType},
    defe,
    network::{ip::{FragFlag, Protocol, ToS, DS, ECN, IP}, getifaddrs},
    throw_errno,
    error::*, data::SAFamily,
};

// #[inline]
// pub unsafe fn create_raw_sock() -> Result<i32> {
//     let raw_sock = throw_errno!(
//         socket(AF_INET, SOCK_RAW, Protocol::Reserved as i32)
//         throws TcpImpError::CreateRawSocketFailed
//     );

//     Ok(raw_sock)
// }

// pub unsafe fn get_send_sock(mut addr: sockaddr_in) -> Result<i32>{
//     let sock = create_raw_sock()?;

//     throw_errno!(bind(
//         sock,
//         &mut addr as *mut sockaddr_in as *mut sockaddr,
//         size_of::<sockaddr_in>() as u32
//     ) throws BindFailed);

//     Ok(sock)
// }
pub const BUF_SIZE_SEND: usize = 16384;
pub const BUF_SIZE_RECV: usize = 131072;

pub static mut BUF_SEND: [u8; BUF_SIZE_SEND] = [0; BUF_SIZE_SEND];
pub static mut BUF_RECV: [u8; BUF_SIZE_RECV] = [0; BUF_SIZE_RECV];

pub static mut IP_ID_ACC: u16 = 0;
pub static mut GEN_IP_ID: fn() -> u16 = || unsafe {
    let old = IP_ID_ACC;
    IP_ID_ACC = IP_ID_ACC.wrapping_add(1);

    old
};

/// (send, recv)
///
/// Both of them bind addr in advance so that we wouldnt use addr in another place.
pub unsafe fn init_socket_pair() -> Result<(i32, i32)> {
    let tx = throw_errno!(
        socket(AF_INET, SOCK_RAW, Protocol::Reserved as i32)
        throws CreateRawSocket
    );

    let rx = throw_errno!(
        socket(AF_INET, SOCK_RAW, Protocol::Reserved as i32)
        throws CreateRawSocket
    );

    let ifaddr = getifaddrs().unwrap().get_sockaddr_in().unwrap();

    let mut addr_local = sockaddr_in {
        sin_family: SAFamily::Inet as u16,
        sin_port: 0,
        sin_addr: in_addr {
            // s_addr: 0,
            s_addr: ifaddr.addr,
        },
        sin_zero: [0; 8],
    };

    throw_errno!(bind(
        tx,
        &mut addr_local as *mut sockaddr_in as *mut sockaddr,
        size_of::<sockaddr_in>() as u32
    ) throws Bind);

    Ok((tx, rx))
}


pub unsafe fn shakehand(sock_send: i32, sock_recv: i32) -> Result<()> {
    // /* Send SYN */
    // let iphdr = IP {
    //     ihl_v: IP::ihl_v(5, 4),
    //     tos: ToS::new(ECN::ECT0, DS::default()).into(),
    //     len: 0,
    //     id: GEN_IP_ID(),
    //     frag_off: IP::frag_off(FragFlag::DF, 0),
    //     ttl: 64,
    //     protocol: Protocol::TCP as u8,
    //     checksum: todo!(),
    //     ip_src: todo!(),
    //     ip_dst: todo!(),
    // };



    Ok(())
}


pub async unsafe fn listen_socket(sock_local: i32, mut addr_remote: sockaddr_in) -> Result<i32> {
    // let epfd = throw_errno!(epoll_create1(0) throws TcpImpError::CreateEpollFailed);

    // let ev = epoll_event {
    //     events: EPOLLIN as u32,
    //     u64: sock as u64,  // union
    // };

    // let mut events: [epoll_event; 1] = [epoll_event { events: 0, u64: 0 }; 1];

    // throw_errno!(epoll_ctl(
    //     epfd,
    //     EPOLL_CTL_ADD,
    //     sock,
    //     &ev as *const epoll_event as *mut epoll_event
    // ) throws TcpImpError::EpollCreateFailed);

    // let _nfds = throw_errno!(epoll_wait(
    //     epfd,
    //     events.as_mut_ptr(),
    //     events.len() as i32,
    //     -1
    // ) throws TcpImpError::EpollWaitFailed);

    let fd = throw_errno!(accept(
        sock_local,
        &mut addr_remote as *mut sockaddr_in as *mut sockaddr,
        &mut size_of::<sockaddr_in>() as *mut usize as *mut socklen_t
    ) throws Accept);


    Ok(fd)
}

// pub unsafe fn listen_socket(sock: i32) -> Result<thread::JoinHandle<()>, std::io::Error> {
//     thread::Builder::new()
//             .name("child-recv".to_owned())
//             .spawn(|| {

//             })
// }

#[cfg(test)]
mod tests {
    use libc::{sockaddr_in, INADDR_ANY, in_addr, INADDR_LOOPBACK};

    use crate::{data::SAFamily, network::{getsockname_sockaddr_in, getifaddrs, ip::Protocol}, aux::htonl};

    use super::init_socket_pair;


    #[test]
    fn test_bind_mechanism() {
        unsafe {
            let addr_remote = sockaddr_in {
                sin_family: SAFamily::Inet as u16,
                sin_port: 8888,
                sin_addr: in_addr {
                    // s_addr: INADDR_ANY,
                    s_addr: htonl(INADDR_LOOPBACK),
                },
                sin_zero: [0; 8],
            };

            let (tx, rx) = init_socket_pair().unwrap();

            let tx_addr = getsockname_sockaddr_in(tx).unwrap();
            let rx_addr = getsockname_sockaddr_in(rx).unwrap();

            println!("tx: {tx_addr:#?}, rx: {rx_addr:#?}");
        }

    }
}