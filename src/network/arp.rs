use std::fmt::Debug;

use super::ip::Protocol;
use crate::{
    aux::htons, data::InAddrN, datalink::Mac, defraw, enum_try_from_int,
};

////////////////////////////////////////////////////////////////////////////////
//// Structure

defraw! {
    /// Address Resolution Protocol (over IPv4)
    pub struct ARP {
        hrd: HT,
        proto: Protocol,
        /// Hardware Length
        hln: u8,
        /// Protocol Address Length
        pln: u8,
        op: ARPOp,
        sha: Mac,
        sip: InAddrN,
        tha: Mac,
        tip: InAddrN
    }

}


/// Hardware Type Network bytes order defined by
/// [IANA](https://www.iana.org/assignments/arp-parameters/arp-parameters.xhtml#arp-parameters-2)
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct HT(pub u16);


enum_try_from_int! {
    #[repr(u16)]
    #[non_exhaustive]
    #[allow(non_camel_case_types)]
    #[derive(Debug)]
    pub enum HTE {
        Reserved0 = 0,
        Ethernet10Mb = 1,
        ExptEher3Mb = 2,
        AmateurRadioAX25 = 3,
        PPTokenRing = 4,
        Chaos = 5,
        IEEE802 = 6,
        ARCNET = 7,
        Hyperchannel = 8,
        Lanstar = 9,
        AutonetShortAddr = 10,
        LocalTalk = 11,
        /// IBM PCNet or SYTEX LocalNet
        LocalNet = 12,
        Ultralink = 13,
        SMDS = 14,
        FrameReply = 15,
        /// Asynchronous Transmission Mode
        ATM16 = 16,
        HDLC = 17,
        FibreChannel = 18,
        ATM19 = 19,
        SerialLine = 20,
        ATM21 = 21,
        MIL_STD_188_220 = 22,
        Metricom = 23,
        IEEE1394_1995 = 24,
        MAPOS = 25,
        Twinaxial = 26,
        EUI64 = 27,
        HIPARP = 28,
        IPARPISO78163 = 29,
        ARPSec = 30,
        IPSecTunnel = 31,
        InfiniBand = 32,
        TIA102ProjInf = 33,
        WiegandInf = 34,
        PureIP = 35,
        HWEXP1 = 36,
        HFI = 37,
        UnifiedBus = 38,

        // 39-255 Unassigned

        AEth = 257,

        // 258-65534 Unassigned

        ReservedFF = 0xFF
    }
}


#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ARPOp(pub u8);

enum_try_from_int! {
    #[repr(u8)]
    #[derive(Debug)]
    pub enum ARPOpE {
        Request = 1,
        Reply = 2
    }
}



////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl Debug for HT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match HTE::try_from(unsafe { htons(self.0) }) {
            Ok(enum_) => write!(f, "{enum_:?}"),
            Err(err) => write!(f, "Unassigned({err})"),
        }
    }
}

impl Debug for ARPOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match ARPOpE::try_from(self.0) {
            Ok(enum_) => write!(f, "{enum_:?}"),
            Err(err) => write!(f, "Invalid ({err})"),
        }
    }
}
