use std::net::Ipv4Addr;

use crate::aux::From2;

pub mod ip;
pub mod icmp;
mod icmp_spec;

use libc::in_addr;


pub unsafe fn inet_cksum(mut data: *const u8, mut len: u32) -> u16 {
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

    sum = (sum >> 16) + (sum & 0xffff);
    sum += sum >> 16;

    !(sum as u16)
}



impl From2<in_addr> for Ipv4Addr {
    fn from2(val: in_addr) -> Self {
        Ipv4Addr::from(val.s_addr)
    }
}


#[cfg(test)]
mod tests {
    use crate::network::inet_cksum;


    #[test]
    fn test_cksum() {
        unsafe {
            let input = [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9];
            let cksum = inet_cksum(input.as_ptr(), input.len() as u32);

            println!("cksum: {:0X} {:0X}", cksum & 0x00ff, (cksum >> 8) & 0x00ff);
        }

    }
}