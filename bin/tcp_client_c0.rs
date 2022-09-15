#![allow(unused_imports)]
#![allow(unused)]

use std::{
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddrV4},
    ptr::{read, write},
    thread::sleep,
};

use chrono::Duration;
use clap::Parser;
// use libc::*;

////////////////////////////////////////////////////////////////////////////////
//// TcpRequest

///
/// 0 - ip
///
///
///
///

// unsafe fn handle_conn(mut stream: TcpStream) {
//     let from_addr = stream.local_addr().unwrap();
//     let mut tcpid = [0u8; 16];
//     match stream.read_exact(&mut tcpid) {
//         Ok(_) => {
//             let id = read(tcpid[..4].as_ptr() as *const u32);
//             println!("{from_addr}: {id}");
//         },
//         Err(err) => eprintln!("{from_addr}: {err:#?}"),
//     }
// }

unsafe fn conn(sockaddrv4: SocketAddrV4) -> std::io::Result<()> {
    // let mut stream = TcpStream::connect(sockaddrv4)?;

    // let mut tcpid = [0u8; 16];
    // let mut recv_buf = [0u8; 1024];
    // write(tcpid[..4].as_mut_ptr() as *mut u32, 2);
    // stream.write(&mut tcpid)?;

    // stream.read_exact(&mut recv_buf[..3])?;

    // let echo = String::from_iter(recv_buf[..3].iter().map(|x| *x as char));

    // println!("recv: {}", echo);
    // // stream.shutdown(Shutdown::Both)?;
    // sleep(Duration::seconds(400).to_std().unwrap());

    Ok(())
}



////////////////////////////////////////////////////////////////////////////////
//// CLI

/// Tcp Server R0
#[derive(Parser)]
#[clap()]
struct Cli {
    #[clap(default_value_t = Ipv4Addr::new(127, 0, 0, 1))]
    ip: Ipv4Addr,

    #[clap(default_value_t = 8888)]
    port: u16,
}



fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let sockaddrv4 = SocketAddrV4::new(cli.ip, cli.port);
    println!("connect to {:?}", sockaddrv4);

    loop {
        unsafe { conn(sockaddrv4)? };
        sleep(Duration::seconds(1).to_std()?)
    }

    // Ok(())
}



#[cfg(test)]
mod tests {

    #[test]
    fn calc_spec_ip_cksum() {


    }

}