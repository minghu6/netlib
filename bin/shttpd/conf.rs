use std::{
    collections::BTreeMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use netlib::error::*;
use serde_yaml::{self, Value};
use getset::{ Getters, Setters };

pub const CONF_NAME_CGIROOT: &str = "cgiroot";
pub const CONF_NAME_DEFAULT_FILE: &str = "default-file";
pub const CONF_NAME_LISTEN_PORT: &str = "listen-port";
// pub const CONF_NAME_CLIENT: &str = "client";
// pub const CONF_NAME_CLIENT_MAX: &str = "max";
// pub const CONF_NAME_CLIENT_INIT: &str = "init";
pub const CONF_NAME_TIMEOUT: &str = "timeout";


#[derive(Getters, Setters, Debug, Clone)]
#[getset(get = "pub", set = "pub")]
pub struct ServConf {
    cgiroot: PathBuf,
    default_file: String,
    listen_port: u16,
    // max_client: u32,
    // init_client: u32,
    timeout: u64,
}



pub fn load_default_conf<P: AsRef<Path>>(p: P) -> Result<ServConf> {
    let file = File::open(p).map_err(|err| NetErr::Open(err))?;
    let buf_file = BufReader::new(file);

    let mut map: BTreeMap<String, Value> =
        serde_yaml::from_reader(buf_file)
            .map_err(|_err| NetErr::MalformedYAML)?;

    let cgiroot = loop {
        if let Some(v) = map.remove(CONF_NAME_CGIROOT) {
            match v {
                Value::String(s) => break PathBuf::from(s),
                _ => (),
            }
        }

        return Err(NetErr::YAMLField(CONF_NAME_CGIROOT));
    };

    let default_file = loop {
        if let Some(v) = map.remove(CONF_NAME_DEFAULT_FILE) {
            match v {
                Value::String(s) => break s,
                _ => (),
            }
        }

        return Err(NetErr::YAMLField(CONF_NAME_DEFAULT_FILE));
    };

    let listen_port = loop {
        if let Some(v) = map.remove(CONF_NAME_LISTEN_PORT) {
            if let Some(n) = v.as_u64() {
                if n <= u16::MAX as u64 {
                    break n as u16;
                }
            }
        }

        return Err(NetErr::YAMLField(CONF_NAME_LISTEN_PORT));
    };

    // let mut clientv = loop {
    //     if let Some(v) = map.remove(CONF_NAME_CLIENT) {
    //         match v {
    //             Value::Mapping(map) => break map,
    //             _ => (),
    //         }
    //     }
    //     return Err(NetErr::YAMLField(CONF_NAME_CLIENT));
    // };

    // let max_client = loop {
    //     if let Some(v) = clientv.remove(CONF_NAME_CLIENT_MAX) {
    //         if let Some(n) = v.as_u64() {
    //             break n as u32;
    //         }
    //     }

    //     return Err(NetErr::YAMLField(CONF_NAME_CLIENT_MAX));
    // };

    // let init_client = loop {
    //     if let Some(v) = clientv.remove(CONF_NAME_CLIENT_INIT) {
    //         if let Some(n) = v.as_u64() {
    //             break n as u32;
    //         }
    //     }

    //     return Err(NetErr::YAMLField(CONF_NAME_CLIENT_INIT));
    // };

    let timeout = loop {
        if let Some(v) = map.remove(CONF_NAME_TIMEOUT) {
            if let Some(n) = v.as_u64() {
                break n;
            }
        }

        return Err(NetErr::YAMLField(CONF_NAME_TIMEOUT));
    };

    Ok(ServConf {
        cgiroot,
        default_file,
        listen_port,
        // max_client,
        // init_client,
        timeout,
    })
}
