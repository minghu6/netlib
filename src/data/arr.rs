//! Array based types
//!

use std::{mem::zeroed, ffi::CStr, fmt::Debug, str::FromStr, convert::Infallible};
use serde_big_array::BigArray;


/// WARNNIG: There are no extra byte for \0 for c string
#[repr(C)]
#[derive(Clone, Copy, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FixStr<const N: usize>{
    #[serde(with = "BigArray")]
    pub raw: [u8; N]
}



impl<const N: usize> Default for FixStr<N> {
    fn default() -> Self {
        Self { raw: unsafe { zeroed() } }
    }
}

impl<const N: usize> FixStr<N> {
    /// WARNING: It's very dangerous since there are no guarantee on \0 terminator
    ///
    /// It's caller duty for safe use!
    pub unsafe fn as_cstr(&self) -> &CStr {
        let cstr = CStr::from_ptr(self.raw.as_ptr() as *const i8);

        cstr
    }

    pub fn to_string(&self) -> String {
        let mut s = String::new();

        for i in 0..N { s.push(self.raw[i] as char) }

        s
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.raw.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.raw.as_mut_ptr()
    }

}


impl<const N: usize> Debug for FixStr<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl<const N: usize> FromStr for FixStr<N> {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut it = Self::default();
        for (i, c) in (0..N).zip(s.as_bytes()) {
            it.raw[i] = *c;
        }

        Ok(it)
    }
}

