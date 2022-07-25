


////////////////////////////////////////////////////////////////////////////////
//// View Structure

use crate::error::NetError;

/// ICMP type and code
#[derive(Debug, PartialEq, Eq)]
pub enum ICMPType {
    /// 0x00
    EchoReply,

    /// 0x03
    DestinationUnreachable(UnreachCode),

    /// 0x05
    RedirectMessage(RedirectMessageCode),

    /// 0x08
    EchoRequest,

    /// 0x09
    RouterAdvertisement,

    /// 0x0A Router discovery/selection/solicitation
    RouterSolicitation,

    /// 0x0B
    TimeExceeded(TimeExceededCode),

    /// 0x0C Bad IP header
    BadParam(BadParamCode),

    /// 0x0D
    Timestamp,

    /// 0x0E
    TimestampReply,

    /// 0x2B 43,
    ExtendedEchoRequest(ExtendErrorCode),

    Other(u8)
}


#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum UnreachCode {
    /// 0
    DstNetworkUnreachable = 0,
    /// 1
    DstHostUnreachable,
    /// 2
    DstProtocolUnreachable,
    /// 3
    DstPortUnreachable,
    /// 4
    FragRequiredDFFlagset,
    /// 5
    SrcRouteFailed,
    /// 6
    DstNetworkUnknown,
    /// 7
    DstHostUnknown,
    /// 8
    SrcHostIsolated,
    /// 9
    NetworkAdmiProhibited,
    /// 10
    HostAdmiProhibited,
    /// 11
    NetworkUnreachableforToS,
    /// 12
    HostUnreachableforToS,
    /// 13 Communication Administratively Prohibited
    CommunicationAdmiProhibited,
    /// 14
    HostPrecedenceViolation,
    /// 15 Sent by a router when receiving a datagram whose Precedence value (priority)
    /// is lower than the minimum allowed for the network at that time.
    PrecedenceCutOff
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum RedirectMessageCode {
    RedirectDatagramforNetwork,
    RedirectDatagramforHost,
    RedirectDatagramforToSAndNetwork,
    RedirectDatagramforTosAndNetwork
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum TimeExceededCode {
    TTLExpired,
    FragReassemblyTimeExceeded
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum BadParamCode {
    PtrIndicatesError,
    MissingRequiredOption,
    BadLen
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum ExtendErrorCode {
    /// 0
    NoError,
    /// 1
    MalformedQuery,
    /// 2
    NoSuchInterface,
    /// 3
    NoSuchTableEntry,
    /// 4
    MultipleInterfacesSatisfyQuery
}


////////////////////////////////////////////////////////////////////////////////
//// Implements

impl TryFrom<u8> for UnreachCode {
    type Error = NetError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 15 {
            Err(NetError::InvalidParam)
        }
        else {
            Ok(unsafe { std::mem::transmute(value) })
        }
    }
}

impl TryFrom<u8> for RedirectMessageCode {
    type Error = NetError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 3 {
            Err(NetError::InvalidParam)
        }
        else {
            Ok(unsafe { std::mem::transmute(value) })
        }
    }
}

impl TryFrom<u8> for TimeExceededCode {
    type Error = NetError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 1 {
            Err(NetError::InvalidParam)
        }
        else {
            Ok(unsafe { std::mem::transmute(value) })
        }
    }
}

impl TryFrom<u8> for BadParamCode {
    type Error = NetError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 2 {
            Err(NetError::InvalidParam)
        }
        else {
            Ok(unsafe { std::mem::transmute(value) })
        }
    }
}

impl TryFrom<u8> for ExtendErrorCode {
    type Error = NetError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 4 {
            Err(NetError::InvalidParam)
        }
        else {
            Ok(unsafe { std::mem::transmute(value) })
        }
    }
}

impl Into<u8> for ICMPType {
    fn into(self) -> u8 {
        match self {
            Self::EchoReply => 0x00,
            Self::DestinationUnreachable(_) => 0x03,
            Self::RedirectMessage(_) => 0x05,
            Self::EchoRequest => 0x08,
            Self::RouterAdvertisement => 0x09,
            Self::RouterSolicitation => 0x0A,
            Self::TimeExceeded(_) => 0x0B,
            Self::BadParam(_) => 0x0C,
            Self::Timestamp => 0x0D,
            Self::TimestampReply => 0x0E,
            Self::ExtendedEchoRequest(_) => 0x2B,
            Self::Other(x) => x,
        }
    }
}
