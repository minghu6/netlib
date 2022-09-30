use std::{fmt::Debug, mem::transmute};

use crate::{
    aux::{htons, ntohs},
    data::InAddrN,
    datalink::{EthTypeN, Mac},
    defraw, deftransparent, enum_try_from_int,
    error::NetErr,
    Result,
};


////////////////////////////////////////////////////////////////////////////////
//// Structure

defraw! {
    /// Address Resolution Protocol (over IPv4)
    #[repr(packed)]
    pub struct ARP {
        hrd: ARPHT,
        proto: EthTypeN,
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


deftransparent! {
    /// Hardware Type Network bytes order defined by
    /// [IANA](https://www.iana.org/assignments/arp-parameters/arp-parameters.xhtml#arp-parameters-2)
    ///
    /// Network bytes order
    pub struct ARPHT(u16);

    /// Network bytes order
    pub struct ARPOp(u16);
}


enum_try_from_int! {
    #[repr(u16)]
    #[allow(non_camel_case_types)]
    #[derive(Debug)]
    pub enum ARPHTE {
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

    #[repr(u16)]
    #[derive(Debug)]
    pub enum ARPOpE {
        Request = 1,
        Reply = 2
    }
}



////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl ARPOp {
    pub fn from_native(v: u16) -> Self {
        unsafe { Self(htons(v)) }
    }

    pub fn native(self) -> Result<ARPOpE> {
        ARPOpE::try_from(unsafe { ntohs(self.0) }).or_else(|code| {
            Err(NetErr::AnyWay(format!("unsupported code: {code}")))
        })
    }
}

impl ARPOpE {
    pub fn net(self) -> ARPOp {
        ARPOp(unsafe { transmute(self) })
    }
}

impl Debug for ARPHT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match ARPHTE::try_from(unsafe { htons(self.0) }) {
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

impl ARPHT {
    pub fn from_native(v: u16) -> Self {
        Self(unsafe { htons(v) })
    }

    pub fn native(self) -> Result<ARPHTE> {
        ARPHTE::try_from(unsafe { ntohs(self.0) }).or_else(|code| {
            Err(NetErr::AnyWay(format!("unsupported code: {code}")))
        })
    }
}

impl ARPHTE {
    pub fn net(self) -> ARPHT {
        ARPHT::from_native(unsafe { transmute(self) })
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Function


#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::ARP;


    #[test]
    fn test_arp_layout() {
        assert_eq!(
            size_of::<ARP>(),
            28,
            "expect 28 found {}",
            size_of::<ARP>()
        );
    }
}
