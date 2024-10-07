use std::mem::zeroed;

use ifstructs::ifreq;
use libc::{c_void, ioctl, open, IFF_NO_PI, IFF_TUN, O_RDWR};

use crate::{
    dev::{copy_if_name, TUNSETIFF, TUNSETPERSIST}, throw_errno, Result
};


pub unsafe fn open_tun(dev: &str) -> Result<i32> {
    let fd = throw_errno!(
        open(c"/dev/net/tun".as_ptr(), O_RDWR)
        throws COpen
    );

    let mut ifr: ifreq = zeroed();
    copy_if_name(ifr.ifr_name.as_mut_ptr(), dev);

    // If flag IFF_NO_PI is not set each frame format is:
    //  Flags [2 bytes]
    //  Proto [2 bytes]
    //  Raw protocol(IP, IPv6, etc) frame
    let ifr_flags = IFF_TUN | IFF_NO_PI;
    ifr.set_flags(ifr_flags as i16);

    throw_errno!(
        ioctl(fd, TUNSETIFF, &ifr as *const ifreq as *const c_void)
        throws CIOCtl
    );

    throw_errno!(
        ioctl(fd, TUNSETPERSIST, 1)
        throws CIOCtl
    );

    Ok(fd)
}



#[cfg(test)]
mod tests {
    use std::ptr;

    use libc::read;

    use crate::network::ip::IP;

    use super::open_tun;

    const BUF_LEN: usize = 4 * 1024;

    #[test]
    fn test_tun() {
        unsafe {
            let fd = open_tun("tunx").unwrap();
            // set_tuntap_group(fd, 1000).unwrap();

            let mut buf = [0u8; BUF_LEN];
            let bufp = buf.as_mut_ptr();

            loop {
                // Read a frame:
                let count = read(fd, bufp as _, BUF_LEN);

                if count < 0 {
                    eprintln!("read failed");
                }
                else {
                    println!("read {count}");
                }

                let iph: IP = ptr::read(bufp as _);

                println!("iph: {:#?}", iph);
            }
        }
    }
}

