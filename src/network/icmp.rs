use std::error::Error;

use getset::{ CopyGetters, Setters };
use serde::{ Deserialize, Serialize };

use crate::aux::{htons, ntohs};

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
    ///
    /// seq: sequence id
    pub fn un_as_echo(id: u16, seq: u16) -> u32 {
        unsafe {
            htons(id) as u32 | (htons(seq) as u32) << 16
        }
    }

    /// -> (id, seq)
    pub fn get_idseq(&self) -> (u16, u16) {
       unsafe {(
        ntohs((self.un & 0xffff) as u16),
        ntohs((self.un >> 16) as u16)
       )}
    }

}


#[cfg(test)]
mod tests {
    use crate::network::icmp::ICMPType;

    use super::ICMP;

    #[test]
    fn test_icmp_un() {
        let ty: u8 = ICMPType::EchoRequest.into();
        let code = 0;
        let cksum = 0;
        let id = 12345;
        let seq = 0;
        let un = ICMP::un_as_echo((id & 0xffff) as u16, seq);

        let icmp = ICMP {
            ty,
            code,
            cksum,
            un,
        };

        let (gid, gseq) = icmp.get_idseq();

        println!("id: {}, seq: {}", id, seq);
        println!("gid: {}, gseq: {}", gid, gseq);

    }
}
