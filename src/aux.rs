
////////////////////////////////////////////////////////////////////////////////
//// Macros

use libc::{c_char, in_addr_t, in_addr};

/// use bincode::{ Options, options };
#[macro_export]
macro_rules! bincode_options {
    () => {
        options().with_fixint_encoding().allow_trailing_bytes()
    };
}

/// 直接从libc::s! 偷了

#[macro_export]
macro_rules! __item {
    ($i:item) => {
        $i
    };
}


#[macro_export]
macro_rules! defe {
    ($($(#[$attr:meta])* pub $t:ident $i:ident { $($field:tt)* })*) => ($(
        defe!(it: $(#[$attr])* pub $t $i { $($field)* });
    )*);
    // (it: $(#[$attr:meta])* pub union $i:ident { $($field:tt)* }) => (
    //     compile_error!("unions cannot derive extra traits, use s_no_extra_traits instead");
    // );
    (it: $(#[$attr:meta])* pub $t:ident $i:ident { $($field:tt)* }) => (
        __item! {
            #[derive(Debug)]
            $(#[$attr])*
            pub $t $i { $($field)* }
        }
        impl std::fmt::Display for $i {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:#?}", self)
            }
        }

        impl std::error::Error for $i {}
    );
}

#[macro_export]
macro_rules! cstr {
    ($val:literal) => {
        CString::new($val).unwrap().as_ptr()
    };
}



pub trait From2<T> {
    fn from2(_: T) -> Self;
}



////////////////////////////////////////////////////////////////////////////////
//// Extern Reference (POSIX)


extern "C" {
    pub fn htons(hostshort: u16) -> u16;
    pub fn ntohs(netshort: u16) -> u16;
    pub fn htonl(hostlong: u32) -> u32;
    pub fn ntohl(netlong: u32) -> u32;
    pub fn inet_addr(cp: *const c_char) -> in_addr_t;
    pub fn inet_ntoa(r#in: in_addr) -> *mut c_char;
}

