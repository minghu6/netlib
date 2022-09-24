use crate::{defraw, view::Hex8, network::ip::Protocol};


////////////////////////////////////////////////////////////////////////////////
//// Structure

defraw! {
    #[repr(packed)]
    pub struct Eth {
        dst: Mac,
        src: Mac,
        proto: Protocol
    }

    pub struct Mac ([Hex8; 6])
}


////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl Mac {
    pub fn new(p1: u8, p2: u8, p3: u8, p4: u8, p5: u8, p6: u8) -> Self {
        Self([Hex8(p1), Hex8(p2), Hex8(p3), Hex8(p4), Hex8(p5), Hex8(p6)])
    }
}




#[cfg(test)]
mod tests {
    use std::ptr::write;

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
        println!("{eth:#?}");
    }
}
