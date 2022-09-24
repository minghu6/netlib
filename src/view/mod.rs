use std::fmt::Debug;


#[derive(Clone, Copy, Default)]
#[repr(transparent)]
pub struct Hex8(pub u8);


impl Debug for Hex8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "0x{:02X}", self.0)
        }
        else {
            write!(f, "0x{:02x}", self.0)
        }
    }
}

