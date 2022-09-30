#![feature(box_syntax)]
#![feature(never_type)]
#![feature(local_key_cell_methods)]


mod skbuff;
mod eth;
mod arp;
mod ip;
mod udp;


use std::env;

use clap::Parser;
use log::info;
use eth::NetDevice;
use netlib::error::{LoggerKind, NetErr, Result};


/// Simple UDP/IP Network Protocol Stack
#[derive(Parser)]
#[clap(name = "SIP")]
struct Cli {
    /// If name
    #[clap()]
    r#if: String,
}

fn setup_logger() -> Result<()> {
    /* Logger should be configured first! */
    let mut logconf = log4rs::config::load_config_file(
        "res/sip/log4rs.default.yaml",
        Default::default(),
    )
    .or_else(|err| {
        Err(NetErr::Log4RS(LoggerKind::LoadConfigFailed(format!(
            "{}",
            err
        ))))
    })?;

    if let Ok(levels) = env::var("RUST_LOG") {
        match levels.parse() {
            Ok(level) => {
                logconf.root_mut().set_level(level);
            }
            Err(err) => {
                return Err(NetErr::Log4RS(LoggerKind::LoadConfigFailed(
                    format!("{}", err),
                )));
            }
        }
    }

    log4rs::init_config(logconf).or_else(|err| {
        Err(NetErr::Log4RS(LoggerKind::InvalidEnv(format!("{}", err))))
    })?;

    Ok(())
}


fn main() {
    let cli = Cli::parse();

    setup_logger().unwrap();

    unsafe {
        let dev = NetDevice::init(cli.r#if.as_str()).unwrap();
        info!("dev init: {:#?}", dev);
        loop {
            if dev.input.is_null() {
                eprintln!("dev.input is null");
                return;
            }

            eth::input(&dev).unwrap();
            // (*dev.input)(&dev).unwrap();
        }
    }


}
