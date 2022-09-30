#![allow(unused)]

use std::mem::zeroed;

use crate::skbuff::PCBUDP;


impl PCBUDP {

    pub fn new() -> Self {
        unsafe {
            let mut it: Self = zeroed();
            it.ttl = 255;

            it
        }
    }


}


