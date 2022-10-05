use crate::defe;


defe! {
    pub enum NetErr {
        InvalidParam,

        Deserialize,
        Serialize,
        MalformedYAML,
        YAMLInvalidField(&'static str),
        YAMLNonExistField(&'static str),

        HttpBadReq(HttpKind),
        Log4RS(LoggerKind),

        UnresolvedHost(String),
        MMPoison,

        EpollCreate,
        EpollCtl,
        EpollWait,

        CreateRawSocket,
        GetSockName,
        Accept,
        SendTo,
        RecvFrom,
        GetIfAddrs,
        SocketRaw,
        Bind,
        COpen,
        CIOCtl,

        StreamShutDown(std::io::Error),
        SetStreamOpt(std::io::Error),
        Read(std::io::Error),

        ReadDir(std::io::Error),
        IterDirEntry(std::io::Error),
        AbsolutePath(std::io::Error),
        Open(std::io::Error),
        Write(std::io::Error),
        CreateThreadPool(std::io::Error),
        CreateDirAll(std::io::Error),
        GetIf(String),
        GetGateway(String),
        AnyWay(String),
        Bug(String)
    }
}

#[derive(Debug)]
pub enum HttpKind {
    TooShort(String),
    InvalidReqLn,
    InvalidHeader(String),
    InvalidBody,

    ReqLnNotFound,
    HeaderNotFoundForVersion(String),
    UnRecognizedVerStr(String),
    /// This error should be a bug
    Bug(String)
    // UnSupportedHttpVer(String)
}

#[derive(Debug)]
pub enum LoggerKind {
    LoadConfigFailed(String),
    InvalidEnv(String)
}

pub type Result<T> = std::result::Result<T, NetErr>;
