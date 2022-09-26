use std::{
    net::Ipv4Addr,
    ptr::write, mem::transmute, fmt::Debug
};

use crate::{aux::htons, data::InAddrN, view::U16N, defraw};

pub use super::ip_spec::*;

////////////////////////////////////////////////////////////////////////////////
//// Data Struct


defraw! {
    /// Or IPHdr (Linux Specified) IPv4 Header
    pub struct IP {
        /// ip header len (or internet header length, low 4 bit) and version (high 4 bit)
        ///
        /// ip header: using unit word = 32bit, value 5 is most common cases in real life
        /// means that 5 x 32 = 20 x 8, 20 bytes, and therefore no options.
        /// options field itself can be of maximum 40 bytes, (ihl while be 15 = 60bytes)
        ihl_v: HLV,

        /// type of service
        tos: ToS,

        /// the datagram length.
        ///
        /// the max value are 65536 bytes theoretically, typically however,
        /// the largest size is 1500 bytes.
        len: PL,

        /// packet id, help in the reassembly of packets.
        id: U16N,

        /// fragment offset && frag_flag of the packet in the data stream
        ///
        /// fragment offset is as units of 8 bytes
        frag_off: FragOff,

        /// time to live
        ttl: u8,

        protocol: Protocol,

        /// IP header checksum
        checksum: u16,

        ip_src: InAddrN,
        ip_dst: InAddrN,

        // Options start here ...
    }
}







////////////////////////////////////////////////////////////////////////////////
//// View Struct


impl IP {
    pub fn get_protocol(&self) -> Protocol {
        Protocol::from(self.protocol)
    }

    pub fn get_src_ip(&self) -> Ipv4Addr {
        self.ip_src.into()
    }

    pub fn get_dst_ip(&self) -> Ipv4Addr {
        self.ip_dst.into()
    }

    /// Used for calc checksum for UDP/TCP
    pub unsafe fn write_pseudo_iphdr(&self, buf: &mut [u8], payload_len: u16) {
        let mut p = buf.as_mut_ptr();

        write(p as *mut u32, transmute(self.ip_src));
        p = p.add(4);

        write(p as *mut u32, transmute(self.ip_dst));
        p = p.add(4);

        write(p, 0);
        p = p.add(1);

        // write(p, (htons(self.protocol as u16) >> 8) as u8);
        // bits order is covered by CPU, which is transparent to user
        write(p, self.protocol as u8);
        p = p.add(1);

        write(p as *mut u16, htons(payload_len));

    }


}



#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    use std::{ptr, mem::size_of};

    use crate::{bincode_options, aux::htons};

    use super::{ DS, Protocol, IP };
    use bincode::{ Options, options };

    #[test]
    fn test_view() {

        println!("{:#0b}", DS::CS1 as u8);
        println!("{:#0b}", DS::CS0 as u8);

        let p = Protocol::from(25);

        println!("{:?}", p);
    }

    #[test]
    fn test_lib_bincode_behav() {
        unsafe {
            let ip = IP::default();
            let config = bincode_options!().allow_trailing_bytes();

            let mut buf = [0u8; 40];

            ptr::write(buf.as_mut_ptr() as *mut _, ip);

            let ip2: IP = config.deserialize(&buf[0..]).unwrap();

            println!("ip2: {:#?}", ip2)
        }

    }

    #[test]
    fn test_h_to_n() {
        unsafe {

            for i in 0..256 {
                let i2 = (htons(i as u16) >> 8) as u8;
                println!("{}: {}", i, i2);
            }
        }
    }

}
