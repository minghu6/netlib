#![feature(read_buf)]


mod conf;
mod req;
mod resp;
mod route;
mod worker;


use std::{
    env,
    net::{Ipv4Addr, SocketAddrV4, TcpListener},
    path::PathBuf,
    sync::Arc,
};

use clap::Parser;
use conf::*;
use futures::executor::{self, ThreadPool};
use log::info;
use log4rs;
use netlib::error::*;
use route::RouteResolver;
use worker::*;


async fn do_listen(ctx: Arc<GloablContext>) -> Result<()> {
    let servaddr =
        SocketAddrV4::new(Ipv4Addr::LOCALHOST, *ctx.servconf.listen_port());

    let listener =
        TcpListener::bind(servaddr).or_else(|_err| Err(NetErr::Bind))?;


    let pool =
        ThreadPool::new().or_else(|err| Err(NetErr::CreateThreadPool(err)))?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => pool.spawn_ok(do_work(ctx.clone(), stream)),
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


pub struct GloablContext {
    pub resolver: RouteResolver,
    pub servconf: ServConf,
}


fn main() -> Result<()> {
    let cli = Cli::parse();

    /* Logger should be configured first! */
    let mut logconf = log4rs::config::load_config_file(
        "res/shttpd/log4rs.default.yaml",
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


    let mut servconf =
        crate::conf::load_default_serv_conf("res/shttpd/shttpd.default.yaml")?;
    info!("servconf_default: {:#?}", servconf);

    if let Some(config_path) = cli.config_file {
        let servconf_user = crate::conf::load_serv_conf(config_path)?;
        info!("servconf_user: {:#?}", servconf_user);

        servconf.override_with(servconf_user);
        info!("servconf: {:#?}", servconf);
    }

    let resolver = RouteResolver::init(&servconf)?;

    let global = Arc::new(GloablContext { resolver, servconf });

    executor::block_on(do_listen(global))?;

    Ok(())
}
