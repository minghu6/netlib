#![feature(read_buf)]


mod conf;
mod worker;
mod req;


use std::{
    net::{Ipv4Addr, SocketAddrV4, TcpListener},
    path::PathBuf,
    env,
};

use clap::Parser;
use conf::*;
use worker::*;
use netlib::error::*;
use futures::executor::{ThreadPool, self};
use log::info;
use log4rs;


async fn do_listen<'a>(conf: ServConf) -> Result<()> {
    let servaddr = SocketAddrV4::new(
        Ipv4Addr::LOCALHOST,
        *conf.listen_port()
    );

    let listener = TcpListener::bind(servaddr)
    .or_else(|_err| Err(NetErr::Bind))?;


    let pool = ThreadPool::new()
        .or_else(|err| Err(NetErr::CreateThreadPool(err)))?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                pool.spawn_ok(do_work(conf.clone(), stream))
            },
            Err(err) => {
                eprintln!("{:?}", err);
            }
        }
    }

    Ok(())
}



////////////////////////////////////////////////////////////////////////////////
//// Cli

#[derive(Parser)]
#[clap(name = "SHTTPD")]
struct Cli {
    #[clap(short = 'c')]
    config_file: Option<PathBuf>,
}


fn main() -> Result<()> {
    let _cli = Cli::parse();

    let servconf = crate::conf::load_default_conf("res/shttpd/shttpd.default.yaml")?;
    let mut logconf = log4rs::config::load_config_file(
        "res/shttpd/log4rs.default.yaml",
        Default::default()
    )
    .or_else(|err| Err(NetErr::Log4RS(LoggerKind::LoadConfigFailed(format!("{}", err)))))?;

    if let Ok(levels) = env::var("RUST_LOG") {
        match levels.parse() {
            Ok(level) => {
                logconf.root_mut().set_level(level);
            },
            Err(err) => {
                return
                Err(NetErr::Log4RS(LoggerKind::LoadConfigFailed(format!("{}", err))));
            }
        }
    }

    log4rs::init_config(logconf)
    .or_else(|err|
        Err(NetErr::Log4RS(LoggerKind::InvalidEnv(format!("{}", err))))
    )?;

    info!("{:#?}", servconf);

    executor::block_on(do_listen(servconf))?;

    Ok(())
}
