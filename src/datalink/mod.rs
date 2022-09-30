use std::{
    fmt::{Debug, Display},
    mem::{transmute, size_of},
    str::FromStr,
};

use libc::{c_void, memcpy, memset};

use crate::{
    aux::{htons, ntohs},
    defraw, deftransparent, enum_try_from_int,
    error::NetErr,
    or2s,
    view::Hex8,
    RawResult, Result,
};


////////////////////////////////////////////////////////////////////////////////
//// Structure


deftransparent! {
    pub struct Mac ([Hex8; 6]);
}


defraw! {
    #[repr(packed)]
    pub struct Eth {
        dst: Mac,
        src: Mac,
        proto: EthTypeN
    }

    // #define PACKET_HOST		0		/* To us		*/
    // #define PACKET_BROADCAST	1		/* To all		*/
    // #define PACKET_MULTICAST	2		/* To group		*/
    // #define PACKET_OTHERHOST	3		/* To someone else 	*/
    // #define PACKET_OUTGOING		4		/* Outgoing of any type */
    // #define PACKET_LOOPBACK		5		/* MC/BRD frame looped back */
    // #define PACKET_USER		6		/* To user space	*/
    // #define PACKET_KERNEL		7		/* To kernel space	*/
    // /* Unused, PACKET_FASTROUTE and PACKET_LOOPBACK are invisible to user space */
    // #define PACKET_FASTROUTE	6		/* Fastrouted frame	*/
    #[repr(u8)]
    pub enum PacType {
        #[default]
        Host = 0,
        Broadcast = 1,
        Multicast = 2,
        OtherHost = 3,
        Outgoing = 4,
        Loopback = 5,
        User = 6,
        Kernel = 7,
    }

}


deftransparent! {
    /// Network bytes order
    pub struct EthTypeN(u16);
}


enum_try_from_int! {
    /// Assigned by https://www.iana.org/assignments/ieee-802-numbers/ieee-802-numbers.xhtml
    /// and linux definition
    #[repr(u16)]
    #[derive(Clone, Copy, Default, Debug)]
    #[non_exhaustive]
    pub enum EthTypeE {
        #[default]
        Zero = 0x0000,
        P8023 = 0x0001,
        PAll = 0x0003,
        IPv4 = 0x0800,
        ARP = 0x0806,
        /// Audio Video Transport Protocol
        AVTP = 0x22F0,
        IPv6 = 0x86DD,
        /// Ethernet flow control
        EthFlowCtrl = 0x8808,
    }
}



////////////////////////////////////////////////////////////////////////////////
//// Implementation


impl EthTypeN {
    pub fn new(tye: EthTypeE) -> Self {
        Self(unsafe { htons(transmute(tye)) })
    }

    pub fn val(self) -> u16 {
        self.0
    }

    pub fn native(self) -> Result<EthTypeE> {
        EthTypeE::try_from(unsafe { ntohs(self.0) })
            .or_else(|code| Err(NetErr::AnyWay(format!("Unsupported 0x{code:0x}"))))
    }
}


impl EthTypeE {
    pub fn net(self) -> EthTypeN {
        EthTypeN::new(self)
    }
}

impl Debug for EthTypeN {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match EthTypeE::try_from(unsafe { ntohs(self.0) }) {
            Ok(enum_) => write!(f, "{enum_:?}"),
            Err(code) => write!(f, "Unsupported(0x{code:02X})"),
        }
    }
}

impl Mac {
    pub fn new(p1: u8, p2: u8, p3: u8, p4: u8, p5: u8, p6: u8) -> Self {
        Self([Hex8(p1), Hex8(p2), Hex8(p3), Hex8(p4), Hex8(p5), Hex8(p6)])
    }

    pub fn broadcast() -> Self {
        Self::new(0xff, 0xff, 0xff, 0xff, 0xff, 0xff)
    }

    pub fn from_slice<T: Copy>(src: &[T]) -> Self {
        let mut arr = [Hex8(0); 6];

        unsafe {
            memcpy(arr.as_mut_ptr() as *mut _, src.as_ptr() as *const _, 6);
        }

        Self(arr)
    }

    pub fn into_arr8(self) -> [u8; 8] {
        let mut arr8 = [0u8; 8];

        unsafe {
            memcpy(
                arr8.as_mut_ptr() as *mut c_void,
                &self as *const Mac as *const c_void,
                6,
            );
        }

        arr8
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|x| x.0 == 0 )
    }

    pub fn clear(&mut self) {
        unsafe {
            memset(self.0.as_mut_ptr() as *mut _, 0, size_of::<Self>());
        }
    }

}


impl Debug for Mac {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}:{:?}:{:?}:{:?}:{:?}:{:?}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5],
        )
    }
}

impl Display for Mac {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for Mac {
    type Err = String;

    fn from_str(s: &str) -> RawResult<Self, Self::Err> {
        let mut mac = Mac::default();

        for (i, subs) in s.split(":").enumerate() {
            mac.0[i] = Hex8(or2s!(u8::from_str_radix(subs, 16))?);
        }

        Ok(mac)
    }
}



#[cfg(test)]
mod tests {
    use std::{mem::size_of, ptr::write};

    use crate::datalink::{Eth, Mac};

    #[test]
    fn test_layout() {
        #[repr(packed)]
        #[derive(Default, Clone, Copy, Debug)]
        struct A {
            a0: [u8; 3],
            a1: A1,
        }

        #[repr(C)]
        #[derive(Default, Clone, Copy, Debug)]
        struct A1 {
            aa: u16,
        }

        let mut a = A::default();
        a.a0[0] = 4;
        a.a1 = A1 { aa: 16 };

        unsafe {
            let ap = &mut a as *mut A as *mut u8;
            write(ap, 7);
            write(ap.add(3), 12);
        }

        println!("{a:#?}");


        let mut eth = Eth::default();
        eth.src = Mac::new(0x00, 0x0c, 0x29, 0x73, 0x9d, 0x15);

        unsafe {
            let ethp = &mut eth as *mut Eth as *mut u8;

            for i in 0..6 {
                write(ethp.add(i), 0xFF);
            }
        }
        println!("{eth:#?}, {} bytes", size_of::<Eth>());
    }
}
