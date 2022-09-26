use std::fmt::Debug;

use crate::{deftransparent, aux::{ntohs, htons}};


deftransparent! {
    pub struct Hex8(u8);
    pub struct U16N(u16);
}



impl U16N {
    pub fn from_native(v: u16) -> Self {
        Self (unsafe { htons(v) })
    }

    pub fn native(&self) -> u16 {
        unsafe { ntohs(self.0) }
    }
}

impl Debug for U16N {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.native())
    }
}


impl Debug for Hex8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{:02X}", self.0)
        }
        else {
            write!(f, "{:02x}", self.0)
        }
    }
}

