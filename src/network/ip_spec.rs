
use std::{
    mem::transmute, fmt::Debug
};

use crate::{aux::{ntohs, htons}, defraw, view::U16N, deftransparent};


deftransparent! {
    /// IP Header Len and Version
    pub struct HLV(u8);

    pub struct ToS(u8);

    /// Network bytes order
    pub struct FragOff(u16);

    pub struct PL(U16N);
}

/// LSB
#[repr(u8)]
#[derive(Debug)]
pub enum FragFlag {
    /// 0b010, Don't Fragmentation
    DF = 0b010,
    /// 0b001
    MF = 0b001,
    /// 0b000, Obey Fragmentation
    OF = 0b000
}


defraw! {
    #[repr(u8)]
    pub enum Protocol {
        #[default]
        /// 0x00 IPv6 Hop by Hop Options
        HopOpt,

        /// 0x01 Internet Control Message Protocol
        ICMP,

        /// 0x02 Interet Group Management Protocol
        ///
        /// used by hosts and adjacent routers on IPv4 networks
        /// to establish multicast group memberships.
        ///
        /// (on IPv6. it's MLD)
        IGMP,

        /// 0x03 Obsolete, give way to EGP.
        ///
        /// Gateway-to-Gateway Protocol,
        GGP,

        /// 0x04 Encapulate IP packet into another IP packet
        IPinIP,

        /// 0x05 Internet Stream Protocol (IPv5, obsolete)
        ST,

        /// 0x06
        TCP,

        /// 0x07 Core-based trees
        ///
        /// a proposal for making IP Multicast scalable
        /// by constructing a tree of routers.
        CBT,

        /// 0x08 Replacement of GGP
        EGP,

        /// 0x09 Interior gateway protocol
        ///
        /// Routing protocol used for exchanging routing table information
        /// between gateways within an autonomous system.
        ///
        /// Including distance-vector routing protocols, link-state routing protocols
        ///
        IGP,

        /// 0x0A BBN RCC Monitoring
        BbnRccMonitoring,

        /// 0x0B Network Voice Protocol v2
        NVP2,

        /// 0x0C PARC Universal Packet, one of two earliest internetworking protocol suite.
        PUP,

        /// 0x0D
        ARGUS,
        /// 0x0E
        EMCON,

        /// 0x0F Cross Net Debugger
        XNET,

        /// 0x10 Chaosnet, early developed local area network technology
        CHAOS,

        /// 0x11
        UDP,

        /// 0x12 Multiplexing
        MUX,

        /// 0x13 DCN Measurement Subsystems
        DCNMeas,

        /// 0x14 Host Monitoring Protocol
        ///
        /// an obsolete TCP/IP protocol
        HMP,

        /// 0x15 Packet Radio Measurement
        PRM,

        /// 0x16 XEROX NS IDP
        XNSIDP,

        /// 0x17
        Trunk1,
        /// 0x18
        Trunk2,
        /// 0x19
        Leaf1,
        /// 0x1A
        Leaf2 = 0x1A,

        /// 0x1B Reliable Data Protocol
        ///
        /// provide facilities for remote loading, debuging
        /// and bulking transfer of images and data.
        ///
        /// Transport Layer Protocol, only experimental implementations for BSD exist
        RDP,

        /// 0x1C Iternet Reliable Transaction Protocol
        IPTP,

        /// 0x1D ISO Transport Protocol Class 4
        ISOTP4,

        /// 0x1E Bulk Data Transfer Protocol
        NETBLT,

        /// 0x1F MFE Networking Services Protocol
        MFENSP,

        /// 0x20 MERIT Internodal Protocol
        MERITINP,

        /// 0x21 Datagram Congestion Control Protocol
        DCCP,

        /// 0x22 Third Party Connection Protocol
        ThirdPC,

        /// 0x23 Inter-Domain Policy Routing Protocol
        IDPR,

        /// 0x24 Xpress Transport Protocol
        ///
        /// Transport layer protocol, developed to replace TCP
        XTP,

        /// 0x25 Datagram Delivery Protocol
        ///
        /// member of the Apple Talk networking protocol suite, Its main responsibility
        /// is for socket-to-socket delivery of datagrams over AppleTalk network.
        DDP,

        /// 0x26 IDPR Control Message Transport Protocol
        IDPRCMTP,

        /// 0x27
        TPPlusPlus,

        /// 0x28 Internet Link, similiar but much simpler than TCP
        IL,

        /// 0x29 IPv6 Encapsulation
        IPv6,

        /// 0x2A Source Demand Routing Protocol
        SDRP,

        /// 0x2B Routing Header for IPv6
        IPv6Route,

        /// 0x2C Fragment Header for IPv6
        IPv6Frag,

        /// 0x2D Inter-Domain Routing Protocol
        IDRP,

        /// 0x2E Resource Reservation Protocol
        RSVP,

        /// 0x2F Generic Routing Encapsulation
        ///
        /// Developed by Cisco system that encapsulates a wide variety
        /// of network layer protocols inside virtual point2point links or
        /// point2multipoint links.
        GRE,

        /// 0x30 Dynamic Source Routing Protocol
        ///
        /// routing protocol for wireless mesh networks
        DSR,

        /// 0x31 Burroughs Network Architecure
        BNA,

        /// 0x32 Encapsulating Security Payload
        ESP,

        /// 0x33 Authentication Header
        AH,

        /// 0x34 Integrated Net Layer Security Protocol
        INLSP,

        /// 0x35 swlPe IP Security Protocol
        SWIPE,

        /// 0x36 NBMA Address Resolution Protocol
        NARP,

        /// 0x37 IP Mobility
        ///
        /// Makes mobile device move from one network
        /// to another mantaining a permanent IP address.
        MOBILE,

        /// 0x38 Transport Layer Security Protocol (TLS)
        TLSP,

        /// 0x39 Simple Key-Management for Internet Protocol
        SKIP,

        /// 0x3A ICMP for IPv6
        IPv6ICMP,

        /// 0x3B No Next Header for IPv6
        IPv6NoNxt,

        /// 0x3C Destination Options for IPv6
        IPv6Opts,

        /// 0x3D Any Host Internet Protocol
        AnyHostIP,

        /// 0x3E
        CFTP,

        /// 0x3F Any Local Network
        AnyLocalNet,
        /// 0x40 SATNET and Backroom EXPAK
        SATEXPACT,
        /// 0x46 VISA Protocol
        VISA,
        /// 0x47 Internet Packet Core Utility
        IPCU,
        /// 0x48 Computer Protocol Network Executive
        CPNX,
        /// 0x49 Computer Protocol Heart Beat
        CPHB,
        /// 0x4A Wang Span Network
        WSN,
        /// 0x4B Packet Video Protocol
        PVP,
        /// 0x4C Backroom SATNET Monitoring
        BrSatMon,
        /// 0x4D SUN ND Protocol-Temporary
        SunNd,
        /// 0x4E Wideband Monitoring
        WbMon,
        /// 0x4F Wideband Expack
        WbExpak,
        /// 0x50 International Organization for Standardization Internetr Protocol
        ISOIP,
        /// 0x51 Versatile Message Transaction Protocol
        VMTP,
        /// 0x52 Secure Versatile Message Transaction Protocol
        SecureVMTP,
        /// 0x53 VINES
        VINES,
        /// 0x54
        ///
        /// TTP, Time-Triggered Protocol, computer network protocol for control systems
        ///
        /// IPTM, Internet Protocol Traffic Manager
        ///
        /// TTP or IPTM, It depends.
        TTPOrIPTM,
        /// 0x55 NSFNET-IGP
        NSFNetIGP,

        /// 0x56 Dissimiliar Protocol Traffic Manager
        DGP,
        /// 0x57
        TCF,
        /// 0x58 Enhanced Interior Gateway Routing Protocol,
        ///
        /// advanced distance-vector routing protocol used for automating routing
        /// decisions and configuration. From Cisco System proprietary protocol
        /// to Open Standard.
        EIGRP,

        /// 0x59 Open Shortest Path First, routing protocol for Internet Protocol networks.
        OSPF,
        /// 0x5A Sprite RPC Protocol
        SpriteRPC,
        /// 0x5B Locus Address Resolution Protocol
        LARP,

        /// 0x5C Multiacst Transport Protocol
        MTP,

        /// 0x5D data link layer protocol (such asunder the IPv4/TCP)
        ///
        /// AX.25 has most frequently been used to establish direct, point-to-point
        /// links between packet radio stations, without any additional network layers.
        AX25,
        /// 0x5E KA9Q NOS compatiable IP over IP tunneling
        OS,
        /// 0x5F Mobile internetworking Control Protocol
        MICP,
        /// 0x60 Semaphore Communications Sec. Pro
        SCCSP,
        /// 0x61 Ethernet within IP Encapsulation
        EtheRip,
        /// 0x62 Encapsulation Header
        EnCap,
        /// 0x63
        AnyPrivateEncryptionScheme,
        /// 0x64
        GMTP,
        /// 0x65 Ipsilon Flow Management Protocol
        IFMP,
        /// 0x66 PNNI over IP
        PNNI,

        /// 0x67 Protocol Independent Multicast
        ///
        /// a family of multicast routing protocols for Internet Protocol (IP) networks
        /// that provide one-to-many and many-to-many distribution of data over a LAN,
        /// WAN or the Internet.
        PIM,

        /// 0x68 IBM's ARIS (Aggregate Route IP Switching) Protocol
        ARIS,
        /// 0x69 Space Communications Protocol Standards
        SCPS,
        /// 0x6A
        QNX,
        /// 0x6B Active Networks
        AN,
        /// 0c6C IP Payload Compression Protocol
        ///
        /// low level compression protocol for IP datagrams, can work with both TCP and UDP
        IPComp,
        /// 0x6D Sitara Networks Protocol
        SNP,
        /// 0x6E Compaq Peer Protocol
        CompaqPeer,
        /// 0x6F IPX in IP
        IPXinIP,
        /// 0x70 Virtual Router Redundancy Protocol
        ///
        /// Supply creation of virual routers
        VRRP,
        /// 0x71 Pragmatic General Multicast
        PGM,

        /// 0x72 Any 0-hop protocol
        Any0Hop,
        /// 0x73 Layer 2 Tunneling Protocol Version 3
        ///
        /// simplified version of MPLS
        L2TP,
        /// 0x74 D-2 Data Exchange
        DDX,
        /// 0x75 Interactive Agent transfer Protocol
        IATP,

        /// 0x76 Schedule Transfer Protocol
        STP,
        /// 0x77 SpetraLink Radio Protocol
        SRP,
        /// 0x78 Universal Transport Interface Protocol
        UTI,

        /// 0x79 Simple Message Protocol
        SMP,
        /// 0x7A
        SM,

        /// 0x7B Performance Transparency Protocol
        PTP,
        /// 0x7C Intermediate System to Intermediate System Protocol over IPv4
        ISISIPv4,
        /// 0x7D  Flexiable Intra-AS Routing Environment
        FIRE,
        /// 0x7E Combat Radio Transport Protocol
        CRTP,
        /// 0x7F Combat User Datagram
        CRUDP,
        /// 0x80 Service-Specific Connection-Oriented Protocol
        /// in a Multilink and Connectionless Environment
        SSCOPMCE,
        /// 0x81
        IPLT,
        /// 0x82 Secure Packet Shield
        SPS,
        /// 0x83 Private IP Encapsulation within IP
        PIPE,
        /// 0x84 Stream Control Transmission Protocol
        ///
        /// transport layer protocol, providing message oriented for UDP
        SCTP,
        /// 0x85 Fibre Channel
        ///
        /// high-speed data transfer protocol providing in-order, lossless delivery
        /// of raw block data
        FC,
        /// 0x86 Reservation Protocol (RSVP) End-to-End Ignore
        RsvpE2eIgnore,

        /// 0x87 Mobility Header for IPv6
        MobiHdr,

        /// 0x88 Lightweight UDP
        UDPLite,
        /// 0x89 Multiprotocol Label Switching Encapsulated in IP
        MPLSInIP,
        /// 0x8A wireless (mobile) ad hoc network
        Manet,
        /// 0x8B Host Identity Protocol
        ///
        /// HIP separates the end-point identifier and locator roles of IP
        /// addresses.
        /// It introduces a Host Identity (HI) name space, based on a public key security infrastructure.
        HIP,

        /// 0x8C Site Multihoming by IPv6
        Shim6,
        /// 0x8D Wrapped Encapsulating Security Payload
        WESP,
        /// 0x8E Robust Header Compression
        ///
        /// standardized method to compress IP, UDP, UDPLite,
        /// RTP(Relatime Transport Protocol), TCP header
        ROHC,

        /// 0x8F Temporary, IPv6 Segment Routing
        Ethernet,

        // 108 unassigned value

        Unassigned144, Unassigned145, Unassigned146, Unassigned147, Unassigned148,
        Unassigned149, Unassigned150, Unassigned151, Unassigned152, Unassigned153,
        Unassigned154, Unassigned155, Unassigned156, Unassigned157, Unassigned158,
        Unassigned159, Unassigned160, Unassigned161, Unassigned162, Unassigned163,
        Unassigned164, Unassigned165, Unassigned166, Unassigned167, Unassigned168,
        Unassigned169, Unassigned170, Unassigned171, Unassigned172, Unassigned173,
        Unassigned174, Unassigned175, Unassigned176, Unassigned177, Unassigned178,
        Unassigned179, Unassigned180, Unassigned181, Unassigned182, Unassigned183,
        Unassigned184, Unassigned185, Unassigned186, Unassigned187, Unassigned188,
        Unassigned189, Unassigned190, Unassigned191, Unassigned192, Unassigned193,
        Unassigned194, Unassigned195, Unassigned196, Unassigned197, Unassigned198,
        Unassigned199, Unassigned200, Unassigned201, Unassigned202, Unassigned203,
        Unassigned204, Unassigned205, Unassigned206, Unassigned207, Unassigned208,
        Unassigned209, Unassigned210, Unassigned211, Unassigned212, Unassigned213,
        Unassigned214, Unassigned215, Unassigned216, Unassigned217, Unassigned218,
        Unassigned219, Unassigned220, Unassigned221, Unassigned222, Unassigned223,
        Unassigned224, Unassigned225, Unassigned226, Unassigned227, Unassigned228,
        Unassigned229, Unassigned230, Unassigned231, Unassigned232, Unassigned233,
        Unassigned234, Unassigned235, Unassigned236, Unassigned237, Unassigned238,
        Unassigned239, Unassigned240, Unassigned241, Unassigned242, Unassigned243,
        Unassigned244, Unassigned245, Unassigned246, Unassigned247, Unassigned248,
        Unassigned249, Unassigned250, Unassigned251, Unassigned252,

        Test253,
        Test254,

        /// or Raw
        Reserved = 0xFF
    }


    /// Occupy high 6 bits
    ///
    /// CS(Class Selector): xxx_000 (backward compatiable with 3 bit precedence)
    ///
    /// AFxy(Assured Forwarding): xxx_yyy ((priority)_(drop precedence))
    ///
    /// AF3y > AF2y > AF1y
    ///
    /// AFx1 > AFx2 > AFx3
    ///
    /// 1. The Interactive Real-Time Traffic (CS4, used for Video
    /// conferencing and Interactive gaming),
    ///
    /// 1. The Broadcast TV (CS3) for use in a video on demand context, and
    ///
    /// 1. The AF4 Multimedia Conferencing (video conferencing).
    ///
    #[repr(u8)]
    pub enum DS {
        #[default]
        CS0 = 0b000_000,
        CS1 = 0b001_000,
        CS2 = 0b010_000,
        CS3 = 0b011_000,
        CS4 = 0b100_000,
        CS5 = 0b101_000,
        CS6 = 0b110_000,
        CS7 = 0b111_000,

        AF11 = 0b001_01_0,  // 1_1
        AF12 = 0b001_10_0,  // 1_2
        AF13 = 0b001_11_0,  // 1_3

        AF21 = 0b010_01_0,
        AF22 = 0b010_10_0,
        AF23 = 0b010_11_0,

        AF31 = 0b011_01_0,
        AF32 = 0b011_10_0,
        AF33 = 0b011_11_0,

        AF41 = 0b100_01_0,
        AF42 = 0b100_10_0,
        AF43 = 0b100_11_0,

        /// CS5
        EF = 0b101_11_0,

        /// CS5
        ///
        /// Used for Telephony Capacity-Admitted Traffic (专享流量)
        ///
        /// from RFC5865, assigned by IANA(Inernet Assigned Numbers Authority)
        VoiceAdmit = 0b101_10_0
    }


    /// Explicit Congestion Notification (occupies low 2 bits)
    ///
    /// ECT0 vs ECT1, reference: https://www.rfc-editor.org/rfc/rfc3168.html#page-55
    ///
    /// (supply one bit nonce)
    #[repr(u8)]
    pub enum ECN {
        /// Not ECT-Capable Transport
        #[default]
        NotECT = 0,

        /// 0b01
        ECT1,

        /// default ECT value, 0b10
        ECT0,

        /// Congestion Experienced, 0b11
        ///
        /// modify the ECT0 or ETC1 to CE
        CE
    }

}


////////////////////////////////////////////////////////////////////////////////
//// Implements

impl From<u8> for DS {
    fn from(val: u8) -> Self {
        unsafe { std::mem::transmute(val) }
    }
}

impl From<u8> for ECN {
    fn from(val: u8) -> Self {
        unsafe { std::mem::transmute(val) }
    }
}

impl Debug for FragOff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:?}, {} bytes)", self.get_frag_flag(), self.get_frag_off_size())
    }
}

impl FragOff {
    /// frag_off (x8 bytes) && frag_flag
    pub fn new(fragflag: FragFlag, off: u16) -> Self {
        Self(unsafe {
            htons((fragflag as u16) << 13 | off)
        })
    }

    pub fn get_frag_flag(&self) -> FragFlag {
        FragFlag::from(
            unsafe { ((ntohs(self.0) & 0xE000) >> 13) as u8 }
        )
    }

    /// as 8 bytes
    pub fn get_frag_off(&self) -> u16 {
        unsafe { ntohs(self.0) & 0x1FFF  }
    }

    /// as bytes
    pub fn get_frag_off_size(&self) -> u16 {
        self.get_frag_off() * 8
    }
}


impl From<u8> for FragFlag {
    fn from(val: u8) -> Self {
        unsafe { std::mem::transmute(val) }
    }
}


impl ToS {
    pub fn get_ds(&self) -> DS {
        DS::from((self.0 >> 2) & 0x3F)
    }

    pub fn get_ecn(&self) -> ECN {
        ECN::from(self.0 & 0x03)
    }

    pub fn new(ecn: ECN, ds: DS) -> Self {
        Self(ecn as u8 | ((ds as u8) << 2))
    }
}

impl Debug for ToS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:?}, {:?})", self.get_ds(), self.get_ecn())
    }
}

impl Into<u8> for ToS {
    fn into(self) -> u8 {
        unsafe { transmute(self) }
    }
}

impl From<u8> for Protocol {
    fn from(val: u8) -> Self {
        unsafe { std::mem::transmute(val) }
    }
}


impl HLV {
    /// ihl default unit of 4 bytes
    ///
    /// v default 4
    pub fn new(ihl: u8, v: u8) -> Self {
        Self(if cfg!(target_endian="little") {
             ihl | (v << 4)
        }
        else {
            debug_assert!(cfg!(target_endian="big"));

            (ihl << 4) | v
        })
    }

    /// bytes
    pub fn get_hdrsize(&self) -> usize {
        (self.get_ihl() * 4) as usize
    }

    /// number of word (4 bytes)
    pub fn get_ihl(&self) -> u8 {
        if cfg!(target_endian="little") {
            self.0 & 0x0F
        }
       else {
           debug_assert!(cfg!(target_endian="big"));

           self.0 >> 4
       }
    }

    pub fn get_version(&self) -> u8 {
        if cfg!(target_endian="little") {
            self.0 >> 4
        }
        else {
           debug_assert!(cfg!(target_endian="big"));

           self.0 & 0x0F
       }
    }

}


impl Debug for HLV {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(v{},{} bytes)", self.get_version(), self.get_hdrsize())
    }
}

impl PL {
    pub fn native(&self) -> u16 {
        self.0.native()
    }

    pub fn from_native(v: u16) -> Self {
        Self( U16N::from_native(v) )
    }
}

impl Debug for PL {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} bytes", self.native())
    }
}
