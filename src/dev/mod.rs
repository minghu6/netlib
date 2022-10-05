pub mod tun;


use std::ffi::c_int;

use libc::{ IFNAMSIZ, ioctl };

use crate::{aux::rstrncpy, size, throw_errno, Result};

/// C style _IOW
#[macro_export]
macro_rules! iow {
    ($ty:expr, $nr:expr, $sz:ident) => {
        ioctl::iow!($ty, $nr, size!($sz))
    };
}

pub const TUNSETIFF: u64 = iow!(b'T', 202, c_int) as u64;
pub const TUNSETPERSIST: u64 = iow!(b'T', 203, c_int) as u64;
pub const TUNSETOWNER: u64 = iow!(b'T', 204, c_int) as u64;
pub const TUNSETGROUP: u64 = iow!(b'T', 206, c_int) as u64;


pub unsafe fn copy_if_name(ifname: *mut u8, dev: &str) {
    if dev.len() + 1 > IFNAMSIZ {
        eprintln!("dev name is too long, at most {}", IFNAMSIZ - 1);
    }

    rstrncpy(ifname as _, dev as _, IFNAMSIZ);
}

pub unsafe fn set_tuntap_group(fd: i32, group: i32) -> Result<()> {
    throw_errno!(
        ioctl(fd, TUNSETGROUP, group)
        throws CIOCtl
    );

    Ok(())
}




#[cfg(test)]
mod tests {
    use super::TUNSETIFF;


    #[test]
    fn view_tun_config() {

        println!("TUNSETIFF: {TUNSETIFF}");
    }

}

