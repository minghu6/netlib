use std::error::Error;

use getset::{ CopyGetters, Setters };
use serde::{ Deserialize, Serialize };

pub use super::icmp_spec::*;


////////////////////////////////////////////////////////////////////////////////
//// Data Structure

/// ICMP Header
///
/// 8 bytes
///
#[repr(C)]
#[derive(CopyGetters, Setters, Deserialize, Serialize, Debug)]
pub struct ICMP {
    pub ty: u8,
    pub code: u8,
    pub cksum: u16,

    pub un: u32
    // data ...
}


////////////////////////////////////////////////////////////////////////////////
//// Implements

impl ICMP {

    /// CRC16
    pub unsafe fn calc_cksum(mut data: *const u8, mut len: u32) -> u16 {
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

    // pub unsafe fn calc_cksum(mut data: *const u8, mut len: u32) -> u16 {
    //     let mut cksum = 0u16;
    //     for i in (0..len as usize).step_by(2) {
    //         cksum += (((*data.add(i)) as u16) << 8) + (*data.add(i + 1) as u16);
    //     }

    //     if len % 2 > 0 {
    //         cksum += *data.add(len as usize - 1) as u16;
    //     }

    //     cksum
    // }

    pub fn parse_cm_type(&self) -> Result<ICMPType, Box<dyn Error>> {
        Ok(match self.ty {
            0x00 => ICMPType::EchoReply,
            0x03 => ICMPType::DestinationUnreachable(UnreachCode::try_from(self.code)?),
            0x05 => ICMPType::RedirectMessage(RedirectMessageCode::try_from(self.code)?),
            0x08 => ICMPType::EchoRequest,
            0x09 => ICMPType::RouterAdvertisement,
            0x0A => ICMPType::RouterSolicitation,
            0x0B => ICMPType::TimeExceeded(TimeExceededCode::try_from(self.code)?),
            0x0C => ICMPType::BadParam(BadParamCode::try_from(self.code)?),
            0x0D => ICMPType::Timestamp,
            0x0E => ICMPType::TimestampReply,
            0x2B => ICMPType::ExtendedEchoRequest(ExtendErrorCode::try_from(self.code)?),

            x => ICMPType::Other(x)
        })
    }

    /// Set echo struct of union
    ///
    /// id: pid
    /// seq: sequence id
    pub fn un_as_echo(id: u16, seq: u16) -> u32 {
        id as u32 + (seq as u32) << 16
    }

    /// -> (id, seq)
    pub fn get_idseq(&self) -> (u16, u16) {
        ((self.un & 0xffff) as u16, (self.un >> 16) as u16)
    }

}


#[cfg(test)]
mod tests {
    use super::ICMP;


    #[test]
    fn test_crc16() {
        unsafe {
            let input = [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9];
            let cksum = ICMP::calc_cksum(input.as_ptr(), input.len() as u32);

            println!("cksum: {:0X} {:0X}", cksum & 0x00ff, (cksum >> 8) & 0x00ff);
        }

    }
}