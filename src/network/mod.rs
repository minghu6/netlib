use std::mem::{size_of, zeroed};

use libc::{
    getsockname, sockaddr,
    sockaddr_in, socklen_t,
};

use crate::{
    data::SockAddrIn,
    error::*,
    throw_errno,
};


pub mod arp;
pub mod icmp;
mod icmp_spec;
pub mod if_;
pub mod ip;
mod ip_spec;


/// Based from [rfc1071](https://www.rfc-editor.org/rfc/inline-errata/rfc1071.html)
pub unsafe fn inet_cksum(mut data: *const u8, mut len: usize) -> u16 {
    // let mut data = data;
    // let mut len = len;

    // the len should bo more than u16::MAX,
    // so u32 should be able to hold the sum.
    let mut sum = 0u32;
    let is_odd = len & 0x1 > 0;

    // sum every two bytes
    while len & 0xfffe > 0 {
        sum += *(data as *const u16) as u32;
        data = data.add(2);
        len -= 2;
    }

    // add the last one byte for odd len
    if is_odd {
        sum += (((*data as u16) << 8) & 0xff00) as u32;
    }

    // add extra overflow bits to the LSB (which diff from warpping_add)
    sum = (sum & 0xffff) + (sum >> 16);
    // at most overflow once
    sum += sum >> 16;

    // equal to while (sum >> 16) > 0 { sum = (sum & 0xffff) + (sum >> 16) }

    // as form of one's completion
    !(sum as u16)
}


pub unsafe fn getsockname_sockaddr_in(socket: i32) -> Result<SockAddrIn> {
    let mut addr = zeroed::<sockaddr_in>();
    let mut addr_len = size_of::<sockaddr_in>() as socklen_t;

    throw_errno!(getsockname(
        socket,
        &mut addr as *mut sockaddr_in as *mut sockaddr,
        &mut addr_len as *mut socklen_t
    ) throws GetSockName );

    Ok(SockAddrIn::from(addr))
}



#[allow(unused)]
#[cfg(test)]
mod tests {
    use std::mem::transmute;

    use crate::{aux::random_u8, network::inet_cksum};

    const RANDOM_BYTES_LEN: usize = 10000;

    fn random_big_bytes(buf: &mut [u8]) {
        for i in 0..RANDOM_BYTES_LEN {
            buf[i] = random_u8();
        }
    }

    // unsafe fn inet_cksum2(mut data: *const u8, mut len: usize) -> u16 {
    //     let mut sum = 0u16;
    //     let is_odd = len & 0x1 > 0;

    //     while len & 0xfffe > 0 {
    //         sum = sum.wrapping_add(*(data as *const u16));
    //         data = data.add(2);
    //         len -= 2;
    //     }

    //     // add the last one byte for odd len
    //     if is_odd {
    //         sum = sum.wrapping_add(((*data as u16) << 8) & 0xff00);
    //     }

    //     !sum
    // }

    // #[test]
    // fn test_cksum() {
    //     unsafe {
    //         let mut buf = [0u8; RANDOM_BYTES_LEN];

    //         for i in 0..100 {
    //             random_big_bytes(&mut buf);
    //             let cksum = inet_cksum(buf.as_ptr(), buf.len());
    //             let cksum2 = inet_cksum2(buf.as_ptr(), buf.len());

    //             assert_eq!(cksum, cksum2, "{i}:")
    //         }
    //     }
    // }
}
