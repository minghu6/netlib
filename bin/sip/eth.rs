use libc::{IFNAMSIZ, sockaddr};
use netlib::{
    defraw0,
    data::{InAddrN, FixStr}, datalink::{EthTypeN, Mac}
};

use crate::skbuff::SKBuff;



defraw0! {
    pub struct NetDevice {
        name: FixStr<IFNAMSIZ>,
        ip_host: InAddrN,
        ip_netmask: InAddrN,
        ip_broadcast: InAddrN,
        ip_gateway: InAddrN,
        ip_dst: InAddrN,
        type_: EthTypeN,

        /// 从网络设备中获取数据，传入协议栈进行处理
        input: *mut fn(*mut SKBuff, *mut NetDevice) -> u8,
        /// IP模块发送数据时调用，经过ARP模块
        output: *mut fn(*mut SKBuff, *mut NetDevice) -> u8,
        /// ARP模块调用
        linkoutput: *mut fn(*mut SKBuff, *mut NetDevice) -> u8,

        hwa_len: u8,
        hwa: Mac,
        hwa_broadcast: Mac,
        mtu: u8,
        /// Sock descriptor
        sd: i32,
        to: *mut sockaddr
    }
}
