use std::{
    collections::BTreeMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use getset::{Getters, Setters};
use netlib::error::*;
use serde_yaml::{self, Value, Mapping};



////////////////////////////////////////////////////////////////////////////////
//// Constant

pub const CONF_NAME_CGIROOT: &str = "cgiroot";
pub const CONF_NAME_DOCROOT: &str = "docroot";
pub const CONF_NAME_CGIMAP: &str = "cgimapping";
pub const CONF_NAME_CGIMAP_CGI: &str = "cgi";
pub const CONF_NAME_CGIMAP_ROUTE: &str = "route";

pub const CONF_NAME_DEFAULT_FILE: &str = "default-file";
pub const CONF_NAME_LISTEN_PORT: &str = "listen-port";
// pub const CONF_NAME_CLIENT: &str = "client";
// pub const CONF_NAME_CLIENT_MAX: &str = "max";
// pub const CONF_NAME_CLIENT_INIT: &str = "init";
pub const CONF_NAME_TIMEOUT: &str = "timeout";



////////////////////////////////////////////////////////////////////////////////
//// Structure

#[derive(Getters, Setters, Debug, Clone)]
#[getset(get = "pub", set = "pub")]
pub struct ServConf {
    cgimap: CGIMap,
    docroot: PathBuf,
    default_file: String,
    listen_port: u16,
    // max_client: u32,
    // init_client: u32,
    timeout: u64,
}


#[derive(Debug, Clone)]
pub struct CGIMap {
    pub root: PathBuf,
    pub items: Vec<CGIMapItem>,
}


#[derive(Debug, Clone)]
pub struct CGIMapItem {
    pub route: PathBuf,
    pub cgi: PathBuf,
}



////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl CGIMap {
    pub fn get<P: AsRef<Path>>(&self, p: P) -> Option<&CGIMapItem> {
        self
        .items
        .iter()
        .find(|item| item.route == p.as_ref())
    }
}



////////////////////////////////////////////////////////////////////////////////
//// Functions

fn load_cgimapping_items(cgimapping: Vec<Value>) -> Result<Vec<CGIMapItem>> {
    let mut items = vec![];

    for itemv in cgimapping.into_iter() {
        let item = loop {
            match itemv {
                Value::Mapping(map) => {
                    break load_cgimapping_item(map)?;
                },
                _ => ()
            }

            return Err(NetErr::YAMLField(CONF_NAME_CGIMAP));
        };

        items.push(item);
    };

    Ok(items)
}


fn load_cgimapping_item(mut map: Mapping) -> Result<CGIMapItem> {
    let route = loop {
        if let Some(routev) = map.remove(CONF_NAME_CGIMAP_ROUTE) {
            match routev {
                Value::String(s) => {
                    break PathBuf::from(s);
                },
                _ => ()
            }
        }

        return Err(NetErr::YAMLField(CONF_NAME_CGIMAP));
    };

    let cgi = loop {
        if let Some(cgiv) = map.remove(CONF_NAME_CGIMAP_CGI) {
            match cgiv {
                Value::String(s) => {
                    break PathBuf::from(s);
                },
                _ => ()
            }
        }

        return Err(NetErr::YAMLField(CONF_NAME_CGIMAP));
    };

    Ok(CGIMapItem { route, cgi })
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

    let cgimapping = loop {
        if let Some(v) = map.remove(CONF_NAME_CGIMAP) {
            match v {
                Value::Sequence(vec) => break vec,
                _ => (),
            }
        }

        return Err(NetErr::YAMLField(CONF_NAME_CGIMAP));
    };

    let cgiitems = load_cgimapping_items(cgimapping)?;
    let cgimap = CGIMap {
        root: cgiroot,
        items: cgiitems,
    };

    let docroot = loop {
        if let Some(v) = map.remove(CONF_NAME_DOCROOT) {
            match v {
                Value::String(s) => break PathBuf::from(s),
                _ => (),
            }
        }

        return Err(NetErr::YAMLField(CONF_NAME_DOCROOT));
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
        cgimap,
        docroot,
        default_file,
        listen_port,
        // max_client,
        // init_client,
        timeout,
    })
}
