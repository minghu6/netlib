
////////////////////////////////////////////////////////////////////////////////
//// Macros

use std::{net::Ipv4Addr, str::FromStr, error::Error, cell::RefCell};

use either::Either;
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
        $crate::__item! {
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
macro_rules! defraw {
    ($($(#[$attr:meta])* pub $t:ident $i:ident { $($field:tt)* })*) => ($(
        defraw!(it: $(#[$attr])* pub $t $i { $($field)* });
    )*);
    (it: $(#[$attr:meta])* pub union $i:ident { $($field:tt)* }) => (
        compile_error!("unions cannot derive extra traits, use s_no_extra_traits instead");
    );
    (it: $(#[$attr:meta])* pub struct $i:ident { $($field:tt)* }) => (
        $crate::__item! {
            #[derive(Debug)]
            #[repr(C)]
            #[cfg_attr(feature = "extra_traits", derive(Debug, Eq, Hash, PartialEq))]
            #[allow(deprecated, non_camel_case_types)]
            $(#[$attr])*
            pub struct $i { $($field)* }
        }
        #[allow(deprecated)]
        impl Copy for $i {}
        #[allow(deprecated)]
        impl Clone for $i {
            fn clone(&self) -> $i { *self }
        }
    );
}

#[macro_export]
macro_rules! cstr {
    ($val:literal) => {
        CString::new($val).unwrap().as_ptr()
    };
}

#[macro_export]
macro_rules! throw_errno {
    ($call:ident ( $($arg:expr),* ) throws $err:ident) => {
        {
            let ret = $call( $($arg),*);

            if ret == -1 {
                eprintln!("{}: {:?}", stringify!($call), $crate::err::ErrNo::fetch());
                return Err($crate::error::NetErr::$err);
            }

            ret
        }
    };
}

#[macro_export]
macro_rules! s {
    ($lit:expr) => {
        String::from($lit)
    };
}


#[macro_export]
macro_rules! or2s {
    ($expr:expr) => {
        $expr.or_else(|err| Err(format!("{err:?}")))
    };
}


////////////////////////////////////////////////////////////////////////////////
//// Traits



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


////////////////////////////////////////////////////////////////////////////////
//// CLI

defe! {
    pub enum CliError {
        ParseInAddrFailed(String),
        UnresolvedHost(String)
    }
}

pub struct HostOrIPv4(Either<Ipv4Addr, String>);

impl FromStr for HostOrIPv4 {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(c) = s.chars().next() {
            Ok(if c.is_digit(10) {
                HostOrIPv4(Either::Left(Ipv4Addr::from_str(s)?))
            } else {
                HostOrIPv4(Either::Right(s.to_owned()))
            })
        } else {
            Err(box CliError::ParseInAddrFailed("".to_string()))
        }
    }
}

impl TryInto<Ipv4Addr> for HostOrIPv4 {
    type Error = Box<dyn Error>;

    fn try_into(self) -> Result<Ipv4Addr, Self::Error> {
        Ok(match self.0 {
            Either::Left(ip) => ip,
            Either::Right(hostname) => {
                let addrs =
                    dns_lookup::getaddrinfo(Some(&hostname), None, None)
                        .or_else(|_lkerr| {
                            Err(box CliError::UnresolvedHost(hostname))
                        })?
                        .collect::<std::io::Result<Vec<_>>>()
                        .unwrap();

                let addr1st = &addrs[0];

                match addr1st.sockaddr.ip() {
                    std::net::IpAddr::V4(ip) => ip,
                    std::net::IpAddr::V6(_) => unreachable!(),
                }
            }
        })
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Helper Function

///// Random //////////////////////////

pub fn software_random() -> usize {
    #[allow(unused_imports)]
    use rand;
    #[allow(unused_imports)]
    use rand::Rng;

    thread_local! {
        static RNG: RefCell<rand::rngs::ThreadRng>  = RefCell::new(rand::thread_rng());
    }

    RNG.with(|rngcell| rngcell.borrow_mut().gen::<usize>())
}

pub fn random() -> usize {
    software_random()
}

pub fn random_u8() -> u8 {
    (random() % u8::MAX as usize) as u8
}

pub fn random_u16() -> u16 {
    (random() % u16::MAX as usize) as u16
}

pub fn random_u32() -> u32 {
    (random() % u32::MAX as usize) as u32
}

///// Counter //////////////////////////

pub type CounterType = impl FnMut() -> usize;

pub fn gen_counter() -> CounterType {
    _gen_counter(0)
}

fn _gen_counter(init: usize) -> CounterType {
    let mut count = init;

    move || {
        let old_count = count;
        count = count.wrapping_add(1);
        old_count
    }
}

// #[cfg(test)]
// mod tests {

//     #[test]
//     fn test_counter() {

//     }
// }
