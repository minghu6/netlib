use crate::defe;


defe! {
    pub enum NetErr {
        InvalidParam,

        DeserializeFailed,
        SerializeFailed,

        UnresolvedHost(String),
        MMPoisonError,

        EpollCreateFailed,
        EpollCtlFailed,
        EpollWaitFailed,

        CreateRawSocketFailed,
        GetSockNameFailed,
        AcceptFailed,
        SendToFailed,
        GetIfAddrsFailed,
        SocketRawFailed,
        BindFailed,
    }
}

pub type Result<T> = std::result::Result<T, NetErr>;
