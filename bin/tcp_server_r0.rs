use std::{net::{TcpListener, TcpStream, SocketAddrV4, Ipv4Addr, Shutdown}, io::{Read, Write}, ptr::read };

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
fn handle_req_2(stream: &mut TcpStream) -> std::io::Result<()> {
    stream.write_all("2+2".as_bytes())

}


unsafe fn handle_conn(mut stream: TcpStream) -> std::io::Result<()> {
    let from_addr = stream.local_addr().unwrap();

    let mut tcpid = [0u8; 16];
    match stream.read_exact(&mut tcpid) {
        Ok(_) => {
            let id = read(tcpid[..4].as_ptr() as *const u32);
            println!("from {from_addr}: {id}");

            if id == 2 {
                handle_req_2(&mut stream)?;
            }
        },
        Err(err) => eprintln!("from {from_addr}: {err:#?}"),
    }

    stream.shutdown(Shutdown::Write)?;

    Ok(())
}

fn main_loop(sockaddrv4: SocketAddrV4) -> std::io::Result<()> {

    let listener = TcpListener::bind(sockaddrv4)?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => unsafe { handle_conn(stream)?; },
            Err(err) => {
                eprintln!("{:?}", err);
            },
        }
    }

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
    port: u16
}



fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let sockaddrv4 = SocketAddrV4::new(cli.ip, cli.port);
    println!("listen {:?}", sockaddrv4);

    main_loop(sockaddrv4)?;
    Ok(())
}

