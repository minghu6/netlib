use std::collections::HashSet;

use getset::CopyGetters;


////////////////////////////////////////////////////////////////////////////////
//// Data Structure

#[repr(C)]
#[derive(CopyGetters, Debug)]
#[getset(get_copy = "pub")]
pub struct TCP {
    source: u16,
    dest: u16,
    seq: u32,
    ack_seq: u32,

    /// Little Endian
    ///
    /// resl: 4
    /// doff: 4
    /// fin: 1
    /// syn: 1
    /// rst: 1
    /// psh: 1
    /// ack: 1
    /// urg: 1
    /// ece: 1
    /// cwr: 1
    ///
    flags: u16,
    /// Window size
    window: u16,
    /// Checksum
    check: u16,
    /// Urgent Pointer
    urgptr: u16
}


////////////////////////////////////////////////////////////////////////////////
//// View Structure

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum TcpFlag {
    /// Finished flag
    Fin,
    /// Synchronisation flag (connection request)
    Syn,
    /// Reset flag
    Rst,
    /// Push flag, go ahead and send  and send what data it has to the receiving
    /// application even if its buffer hasn't filled up.
    Psh,
    /// Acknowledgment flag
    Ack,
    /// Urgent flag
    Urg,
    /// Explicit Congestion Notification Capable flag
    Ece,
    /// Congestion window reduced flag
    Cwr
}



////////////////////////////////////////////////////////////////////////////////
//// Implements

impl TCP {
    pub fn get_flags(&self) -> HashSet<TcpFlag> {
        let mut flags = HashSet::new();

        let _doff: u8 = ((self.flags & 0x0F00) >> 8) as u8;
        let fin: u8 = ((self.flags & 0x0080) >> 7) as u8;
        let syn: u8 = ((self.flags & 0x0040) >> 6) as u8;
        let rst: u8 = ((self.flags & 0x0020) >> 5) as u8;
        let psh: u8 = ((self.flags & 0x0010) >> 4) as u8;
        let ack: u8 = ((self.flags & 0x0008) >> 3) as u8;
        let urg: u8 = ((self.flags & 0x0004) >> 2) as u8;
        let ece: u8 = ((self.flags & 0x0002) >> 1) as u8;
        let cwr: u8 = ((self.flags & 0x0001) >> 0) as u8;

        if fin > 0 {
            flags.insert(TcpFlag::Fin);
        }
        if syn > 0 {
            flags.insert(TcpFlag::Syn);
        }
        if rst > 0 {
            flags.insert(TcpFlag::Rst);
        }
        if psh > 0 {
            flags.insert(TcpFlag::Psh);
        }
        if ack > 0 {
            flags.insert(TcpFlag::Ack);
        }
        if urg > 0 {
            flags.insert(TcpFlag::Urg);
        }
        if ece > 0 {
            flags.insert(TcpFlag::Ece);
        }
        if cwr > 0 {
            flags.insert(TcpFlag::Cwr);
        }

        flags

    }

}