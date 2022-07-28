use crate::{ defe, __item };



defe! {
    pub enum NetError {
        InvalidParam,
        CreateRawSocketFailed,
        DeserializeFailed,
        SerializeFailed,
        SendToFailed
    }
}
