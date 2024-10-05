////////////////////////////////////////////////////////////////////////////////
//// Macros

use std::{cell::RefCell, error::Error, net::Ipv4Addr, str::FromStr, cmp::min};

use either::Either;
use libc::{c_char, in_addr, in_addr_t, memcpy};

#[macro_export]
macro_rules! bincode_options {
    () => {
        {
            use bincode::{options, Options};

            options()
            .with_fixint_encoding()
            .allow_trailing_bytes()
        }
    };
}


/// `defe` 偷自libc::s!

#[macro_export]
macro_rules! __item {
    ($i:item) => {
        $i
    };
}


#[macro_export]
macro_rules! defe {
    ($($(#[$outter:meta])* pub $t:ident $i:ident { $($field:tt)* })*) => ($(
        defe!(it: $(#[$outter])* pub $t $i { $($field)* });
    )*);
    // (it: $(#[$outter:meta])* pub union $i:ident { $($field:tt)* }) => (
    //     compile_error!("unions cannot derive extra traits, use s_no_extra_traits instead");
    // );
    (it: $(#[$outter:meta])* pub $t:ident $i:ident { $($field:tt)* }) => (
        $crate::__item! {
            #[derive(Debug)]
            $(#[$outter])*
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
    ($(#[$outter:meta])* pub $t:ident $i:ident $($rem:tt)*) => (
        $crate::__defraw!(
            #[derive(Default, Clone, Copy, Debug, Eq, PartialEq, Hash)]
            #[allow(deprecated, non_camel_case_types)]
            $(#[$outter])* pub $t $i $($rem)*
        );
    );
    ($($rem:tt)*) => ();
}


#[macro_export]
macro_rules! __defraw {
    // normal structure
    (
        $(#[$outter:meta])*
        pub struct $i:ident {
            $(
                $( #[$inner:meta] )*
                $field_name:ident : $ty:ty
            ),* $(,)?
        }
        $($rem:tt)*
    ) => (
        #[repr(C)]
        $(#[$outter])*
        pub struct $i {
            $(
                $(#[$inner])*
                pub $field_name : $ty
            ),*
        }

        $crate::defraw!($($rem)*);
    );

    // one field tuple structure
    ($(#[$outter:meta])* pub struct $i:ident ( $ty:ty ); $($rem:tt)*) => (
        #[repr(C)]
        $(#[$outter])*
        pub struct $i ( pub $ty );

        $crate::defraw!($($rem)*);
    );

    // enum
    (
        $(#[$outter:meta])*
        pub enum $i:ident {
            $(
                $(#[$inner:meta])*
                $key:ident $(= $value:expr)?
            ),* $(,)?
        }
        $($rem:tt)*
    ) => (
        $(#[$outter])*
        pub enum $i {
            $(
                $(#[$inner])*
                $key $(= $value)?
            ),*
        }

        $crate::defraw!($($rem)*);
    );

    // // union
    // (
    //     $(#[$outter:meta])*
    //     pub union $i:ident {
    //         $(
    //             $( #[$inner:meta] )*
    //             $field_name:ident : $ty:ty
    //         ),* $(,)?
    //     }
    //     $($rem:tt)*
    // ) => (
    //     $(#[$outter])*
    //     pub union $i {
    //         $(
    //             $(#[$inner])*
    //             pub $field_name : $ty
    //         ),*
    //     }

    //     $crate::defraw!($($rem)*);
    // );

    ($($rem:tt)*) => ();
}


#[macro_export]
macro_rules! deftransparent {
    ($(#[$outter:meta])* pub struct $i:ident ( $ty:ty ) ; $($rem:tt)*) => (
        #[repr(C)]
        #[derive(Clone, Copy, Default, Hash, PartialEq, Eq,
            serde::Serialize, serde::Deserialize,)]
        $(#[$outter])*
        pub struct $i (pub $ty);
        deftransparent!($($rem)*);
    );
    () => ()
}

#[macro_export]
macro_rules! deftransparent1 {
    ($(#[$outter:meta])* pub struct $i:ident ( $ty:ty ) ; $($rem:tt)*) => (
        #[repr(C)]
        #[derive(Clone, Copy)]
        $(#[$outter])*
        pub struct $i (pub $ty);
        deftransparent1!($($rem)*);
    );
    () => ()
}

/// Mini version of defraw
///
/// Just Clone
#[macro_export]
macro_rules! defraw0 {
    (
        $(#[$outter:meta])*
        $vis:vis $t:ident $i:ident {
            $(
                $( #[$inner:meta] )*
                $field_name:ident : $ty:ty
            ),* $(,)?
        }
        $($rem:tt)*
    ) => (
        #[repr(C)]
        $(#[$outter])*
        #[derive(Clone)]
        $vis $t $i {
            $(
                $(#[$inner])*
                $vis $field_name : $ty
            ),*
        }

        defraw0!($($rem)*);
    );

    // enum
    (
        $(#[$outter:meta])*
        pub enum $i:ident {
            $(
                $(#[$inner:meta])*
                $key:ident $(= $value:expr)?
            ),* $(,)?
        }
        $($rem:tt)*
    ) => (
        $(#[$outter])*
        pub enum $i {
            $(
                $(#[$inner])*
                $key $(= $value)?
            ),*
        }

        defraw0!($($rem)*);
    );

    () => ()
}

/// Clone + Copy
#[macro_export]
macro_rules! defraw1 {
    (
        $(#[$outter:meta])*
        $vis:vis $t:ident $i:ident {
            $(
                $( #[$inner:meta] )*
                $field_name:ident : $ty:ty
            ),* $(,)?
        }
        $($rem:tt)*
    ) => (
        #[repr(C)]
        $(#[$outter])*
        #[derive(Clone, Copy)]
        $vis $t $i {
            $(
                $(#[$inner])*
                $vis $field_name : $ty
            ),*
        }

        defraw1!($($rem)*);
    );

    // enum
    (
        $(#[$outter:meta])*
        pub enum $i:ident {
            $(
                $(#[$inner:meta])*
                $key:ident $(= $value:expr)?
            ),* $(,)?
        }
        $($rem:tt)*
    ) => (
        $(#[$outter])*
        pub enum $i {
            $(
                $(#[$inner])*
                $key $(= $value)?
            ),*
        }

        defraw1!($($rem)*);
    );

    () => ()
}


#[macro_export]
macro_rules! enum_try_from_int {
    ($(
        $( #[doc=$outterdoc:literal] )*
        #[repr($T: ident)]
        $( #[$outter2: meta] )*
        $vis: vis enum $Name: ident {
            $(
                $( #[$inner: meta] )*
                $Variant: ident = $value: expr
            ),*
            $(,)?
        }
    )*) => {$(
        $( #[doc=$outterdoc] )*
        #[repr($T)]
        $( #[$outter2] )*
        $vis enum $Name {
            $(
                $( #[$inner] )*
                $Variant = $value
            ),*
        }

        impl std::convert::TryFrom<$T> for $Name {
            type Error = $T;

            fn try_from(value: $T) -> std::result::Result<$Name, Self::Error> {
                match value {
                    $(
                        $value => Ok($Name::$Variant),
                    )*
                    _ => Err(value)
                }
            }
        }
    )*}
}


#[macro_export]
macro_rules! __throw_errno_post {
    ($err:ident $errno:ident withs) => {
        return Err($crate::NetErr::$err(format!("{:?}", $errno)));
    };
    ($err:ident $errno:ident ) => {
        return Err($crate::rs_error::NetErr::$err);
    };
}

#[macro_export]
macro_rules! throw_errno {
    ($call:ident ( $($arg:expr),* ) throws $err:ident $($rem:tt)*) => {
        {
            let ret = $call( $($arg),*);

            if ret == -1 {
                let errno = $crate::c_error::ErrNo::fetch();
                eprintln!("{}: {:?}", stringify!($call), errno);
                $crate::__throw_errno_post!($err errno $($rem)*);
            }

            ret
        }
    };
}

#[macro_export]
macro_rules! cstr {
    ($s:expr) => {
        std::ffi::CString::new($s).unwrap().into_raw()
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

#[macro_export]
macro_rules! or2anyway {
    ($expr:expr) => {
        $expr.or_else(|err| Err($crate::NetErr::AnyWay(format!("{err:?}"))))
    };
}

#[macro_export]
macro_rules! retanyway {
    ($($t:tt)*) => {
        return Err($crate::NetErr::AnyWay(format!($($t)*)));
    };
}


/* Align size */
#[macro_export]
macro_rules! alignsz {
    ($x:expr, $word:expr) => {
        {
            let x = $x;
            let word = $word;

            (x + word - 1) & !(word - 1)
        }
    };
    ($x:expr) => {
        alignsz!($x, 4)
    }
}

/// Shorten version of size_of::<T>()
#[macro_export]
macro_rules! size {
    ($name:ident) => {
        std::mem::size_of::<$name>()
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
            }
            else {
                HostOrIPv4(Either::Right(s.to_owned()))
            })
        }
        else {
            Err(Box::new(CliError::ParseInAddrFailed("".to_string())))
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
                            Err(Box::new(CliError::UnresolvedHost(hostname)))
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

/// Rust version strncpy, copy n-1 bytes and end with '\0'
pub unsafe fn rstrncpy(dst: *mut u8, src: &str, n: usize) {
    let copiedn = min(src.len(), n - 1);

    memcpy(dst as _, src.as_ptr() as _, copiedn);
    (*dst.add(copiedn + 1)) = 0;
}


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

pub fn gen_counter() -> impl FnMut() -> usize {
    _gen_counter(0)
}

fn _gen_counter(init: usize) -> impl FnMut() -> usize {
    let mut count = init;

    move || {
        let old_count = count;
        count = count.wrapping_add(1);
        old_count
    }
}
