use std::fmt::Display;



#[derive(Debug)]
pub enum NetError {
    InvalidParam
}

impl Display for NetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl std::error::Error for NetError {}

