use serde::{Deserialize, Serialize};

use crate::aux::{ntohl, ntohs};

////////////////////////////////////////////////////////////////////////////////
//// Data Structure

#[repr(C)]
#[derive(Debug, Deserialize, Serialize)]
pub struct TCP {
    pub source: u16,
    pub dest: u16,
    pub seq: u32,
    /// When ACK is set,
    /// this field is the next sequence number that the sender of the ACK is expecting
    pub ack_seq: u32,

    /// Little Endian
    /// resl: 4 (or 3 + NS bit)
    ///
    /// doff: 4 (data offset, or in other words, tcp header len, size of 4 bytes, similiar with ip header len
    ///     5 - 15, 20 bytes - 60 bytes, 40 bytes options )
    ///
    /// fin: 1
    ///
    /// syn: 1
    ///
    /// rst: 1
    ///
    /// psh: 1
    ///
    /// ack: 1
    ///
    /// urg: 1
    ///
    /// ece: 1
    ///
    /// cwr: 1
    ///
    pub doff_flags: u16,
    /// Window size
    pub window: u16,
    /// Checksum
    pub check: u16,
    /// Urgent Pointer
    ///
    /// Offset from the seq, indicating the last urgent data byte (urg flag required)
    ///
    /// According to rfc6093, new implementations shouldn't use it
    /// just for back compatiablility.
    pub urgptr: u16,
}


////////////////////////////////////////////////////////////////////////////////
//// View Structure

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
#[cfg(target_endian = "little")]
pub enum TcpFlag {
    /// Finished flag
    Fin = 0b0000_0001,
    /// Synchronisation flag (connection request)
    Syn = 0b0000_0010,
    /// Reset flag
    Rst = 0b0000_0100,
    /// Push flag, go ahead and send  and send what data it has to the receiving
    /// application even if its buffer hasn't filled up.
    Psh = 0b0000_1000,
    /// Acknowledgment flag
    Ack = 0b0001_0000,
    /// Urgent flag
    Urg = 0b0010_0000,
    /// Explicit Congestion Notification Capable flag
    Ece = 0b0100_0000,
    /// Congestion window reduced flag
    Cwr = 0b1000_0000,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum TcpOpt {
    /// 0
    END,

    /// 1
    NOP,

    /// 2
    ///
    /// `wnd * 2^scale, 0<=scale<=14` (syn only)
    WndScale(u8),

    /// 3 (syn only)
    ///
    /// bytes
    MSS(u16),

    /// 4 (syn only)
    EnableSA,

    /// 5, todo:
    SA,

    /// 8
    ///
    /// 4 bytes sender timestamp, 4 bytes reply timestamp (the most recent timestamp received)
    Timestamp(u32, u32),
}


////////////////////////////////////////////////////////////////////////////////
//// Implements

impl TCP {
    pub fn get_src_port(&self) -> u16 {
        unsafe {
            ntohs(self.source)
        }
    }

    pub fn get_dst_port(&self) -> u16 {
        unsafe {
            ntohs(self.dest)
        }
    }

    pub fn get_flags(&self) -> Vec<TcpFlag> {
        let mut flags = Vec::new();

        let _doff: u8 = ((self.doff_flags & 0x0F00) >> 8) as u8;
        let fin: u8 = ((self.doff_flags & 0x0080) >> 7) as u8;
        let syn: u8 = ((self.doff_flags & 0x0040) >> 6) as u8;
        let rst: u8 = ((self.doff_flags & 0x0020) >> 5) as u8;
        let psh: u8 = ((self.doff_flags & 0x0010) >> 4) as u8;
        let ack: u8 = ((self.doff_flags & 0x0008) >> 3) as u8;
        let urg: u8 = ((self.doff_flags & 0x0004) >> 2) as u8;
        let ece: u8 = ((self.doff_flags & 0x0002) >> 1) as u8;
        let cwr: u8 = ((self.doff_flags & 0x0001) >> 0) as u8;

        if fin > 0 {
            flags.push(TcpFlag::Fin);
        }
        if syn > 0 {
            flags.push(TcpFlag::Syn);
        }
        if rst > 0 {
            flags.push(TcpFlag::Rst);
        }
        if psh > 0 {
            flags.push(TcpFlag::Psh);
        }
        if ack > 0 {
            flags.push(TcpFlag::Ack);
        }
        if urg > 0 {
            flags.push(TcpFlag::Urg);
        }
        if ece > 0 {
            flags.push(TcpFlag::Ece);
        }
        if cwr > 0 {
            flags.push(TcpFlag::Cwr);
        }

        flags
    }

    pub fn doff_flags(doff: u8, flags: &[TcpFlag]) -> u16 {
        #[cfg(target_endian = "little")]
        {
            let mut fv = 0u8;

            for flag in flags.iter() {
                fv |= *flag as u8;
            }

            0u16 | (doff as u16) << 4 | (fv as u16) << 8
        }
    }

    /// TCP header len (doff*4) bytes
    pub fn get_hdr_len(&self) -> usize {
        (((self.doff_flags & 0x0F00) >> 8) * 4) as usize
    }

    pub fn has_opt(&self) -> bool {
        ((self.doff_flags & 0x0F00) >> 8) > 5
    }

    pub fn opt_len(&self) -> usize {
        self.get_hdr_len() - 20
    }

    pub unsafe fn read_opt(&self, src: *const u8) -> TcpOpt {
        let optlen = self.opt_len();
        debug_assert!(optlen > 0);

        let kind = *src;

        if optlen == 1 {
            if kind == 0 {
                return TcpOpt::END;
            }
            else if kind == 1 {
                return TcpOpt::NOP;
            }
            else {
                unreachable!("{kind} shoule be 1 or 0")
            }
        }

        debug_assert_eq!(optlen, *src.add(1) as usize);

        if kind == 2 {
            debug_assert_eq!(optlen, 4);
            let mss = ntohs(*(src.add(2) as *const u16));
            TcpOpt::MSS(mss)
        }
        else if kind == 3 {
            debug_assert_eq!(optlen, 3);
            let scale = *src.add(2);
            TcpOpt::WndScale(scale)
        }
        else if kind == 4 {
            debug_assert_eq!(optlen, 2);
            TcpOpt::EnableSA
        }
        else if kind == 5 {
            unimplemented!()
        }
        else if kind == 8 {
            debug_assert_eq!(optlen, 10);
            let timestamp = ntohl(*(src.add(2) as *const u32));
            let reply_timestamp = ntohl(*(src.add(6) as *const u32));
            TcpOpt::Timestamp(timestamp, reply_timestamp)
        }
        else {
            unimplemented!("kind: {kind}")
        }
    }
}
