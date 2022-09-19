use std::{
    collections::BTreeMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf}, convert::TryFrom,
};

use getset::{Getters, Setters};
use netlib::error::*;
use serde_yaml::{self, Mapping, Value};



////////////////////////////////////////////////////////////////////////////////
//// Constant

pub const CONF_NAME_CGIROOT: &str = "cgiroot";
pub const CONF_NAME_DOCROOT: &str = "docroot";
pub const CONF_NAME_CGIMAP: &str = "cgimapping";
pub const CONF_NAME_CGIMAP_CGI: &str = "cgi";
pub const CONF_NAME_CGIMAP_ROUTE: &str = "route";

pub const CONF_NAME_DEFAULT_FILE: &str = "default-file";
pub const CONF_NAME_PERSISROOT: &str = "persisroot";

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
    default_file: PathBuf,
    persisroot: PathBuf,
    listen_port: u16,
    // max_client: u32,
    // init_client: u32,
    timeout: u64,
}

#[derive(Debug)]
pub struct ServConfOpt {
    cgiroot: Option<PathBuf>,
    cgimap: Option<CGIMapOpt>,
    docroot: Option<PathBuf>,
    default_file: Option<PathBuf>,
    persisroot: Option<PathBuf>,
    listen_port: Option<u16>,
    // max_client: u32,
    // init_client: u32,
    timeout: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct CGIMap {
    pub root: PathBuf,
    pub items: Vec<CGIMapItem>,
}

#[derive(Debug)]
pub struct CGIMapOpt {
    items: Vec<CGIMapItem>,
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
        self.items.iter().find(|item| item.route == p.as_ref())
    }

    pub fn getfullpath<P: AsRef<Path>>(&self, p: P) -> Option<PathBuf> {
        self.get(p)
            .map(|cgiitem| cgiitem.joined_by(&self.root))
    }
}


impl CGIMapItem {
    pub fn joined_by(&self, root: &PathBuf) -> PathBuf {
        root.join(trim_path_prefix(&self.cgi, "/"))
    }
}


impl ServConf {
    /// Override if other filed is not empty
    pub fn override_with(&mut self, other: ServConfOpt) {
        let ServConfOpt {
            cgiroot,
            cgimap,
            docroot,
            default_file,
            persisroot,
            listen_port,
            timeout,
        } = other;

        if let Some(cgiroot) = cgiroot {
            self.cgimap.root = cgiroot;
        }

        if let Some(cgimap) = cgimap {
            self.cgimap.items = cgimap.items;
        }

        if let Some(docroot) = docroot {
            self.docroot = docroot;
        }

        if let Some(default_file) = default_file {
            self.default_file = default_file;
        }

        if let Some(persisroot) = persisroot {
            self.persisroot = persisroot;
        }

        if let Some(listen_port) = listen_port {
            self.listen_port = listen_port;
        }

        if let Some(timeout) = timeout {
            self.timeout = timeout;
        }

    }
}

impl TryFrom<ServConfOpt> for ServConf {
    type Error = NetErr;

    fn try_from(value: ServConfOpt) -> Result<Self> {
        let cgiroot = value.cgiroot
        .ok_or(NetErr::YAMLNonExistField(CONF_NAME_CGIROOT))?;

        let cgimap = CGIMap {
            root: cgiroot,
            items: value.cgimap
            .ok_or(NetErr::YAMLNonExistField(CONF_NAME_CGIMAP))?
            .items
        };

        let docroot = value.docroot
        .ok_or(NetErr::YAMLNonExistField(CONF_NAME_DOCROOT))?;

        let default_file = value.default_file
        .ok_or(NetErr::YAMLNonExistField(CONF_NAME_DEFAULT_FILE))?;

        let persisroot = value.persisroot
        .ok_or(NetErr::YAMLNonExistField(CONF_NAME_PERSISROOT))?;

        let listen_port = value.listen_port
        .ok_or(NetErr::YAMLNonExistField(CONF_NAME_LISTEN_PORT))?;

        let timeout = value.timeout
        .ok_or(NetErr::YAMLNonExistField(CONF_NAME_TIMEOUT))?;


        Ok(Self {
            cgimap,
            docroot,
            default_file,
            persisroot,
            listen_port,
            timeout,
        })

    }
}


////////////////////////////////////////////////////////////////////////////////
//// Functions


pub fn trim_path_prefix<'a, P: AsRef<Path>>(path: P, pat: &str) -> PathBuf {
    PathBuf::from(path.as_ref().to_str().unwrap().trim_start_matches(pat))
}


fn load_cgimapping_items(cgimapping: Vec<Value>) -> Result<Vec<CGIMapItem>> {
    let mut items = vec![];

    for itemv in cgimapping.into_iter() {
        let item = loop {
            match itemv {
                Value::Mapping(map) => {
                    break load_cgimapping_item(map)?;
                }
                _ => (),
            }

            return Err(NetErr::YAMLInvalidField(CONF_NAME_CGIMAP));
        };

        items.push(item);
    }

    Ok(items)
}


fn load_cgimapping_item(mut map: Mapping) -> Result<CGIMapItem> {
    let route = loop {
        if let Some(routev) = map.remove(CONF_NAME_CGIMAP_ROUTE) {
            match routev {
                Value::String(s) => {
                    break PathBuf::from(s);
                }
                _ => (),
            }
        }

        return Err(NetErr::YAMLInvalidField(CONF_NAME_CGIMAP));
    };

    let cgi = loop {
        if let Some(cgiv) = map.remove(CONF_NAME_CGIMAP_CGI) {
            match cgiv {
                Value::String(s) => {
                    break PathBuf::from(s);
                }
                _ => (),
            }
        }

        return Err(NetErr::YAMLInvalidField(CONF_NAME_CGIMAP));
    };

    Ok(CGIMapItem { route, cgi })
}


pub fn load_default_serv_conf<P: AsRef<Path>>(p: P) -> Result<ServConf> {
    let servconfopt = load_serv_conf(p)?;

    Ok(ServConf::try_from(servconfopt)?)
}


pub fn load_serv_conf<P: AsRef<Path>>(p: P) -> Result<ServConfOpt> {
    let file = File::open(p).map_err(|err| NetErr::Open(err))?;
    let buf_file = BufReader::new(file);

    let mut map: BTreeMap<String, Value> =
        serde_yaml::from_reader(buf_file)
            .map_err(|_err| NetErr::MalformedYAML)?;

    let cgiroot =
    if let Some(v) = map.remove(CONF_NAME_CGIROOT) {
        Some(match v {
            Value::String(s) => PathBuf::from(s),
            _ => return Err(NetErr::YAMLInvalidField(CONF_NAME_CGIROOT)),
        })
    }
    else {
        None
    };

    let cgimapopt =
    if let Some(v) = map.remove(CONF_NAME_CGIMAP) {
        Some(match v {
            Value::Sequence(vec) => {
                let cgiitems = load_cgimapping_items(vec)?;
                CGIMapOpt {
                    items: cgiitems,
                }
            },
            _ => return Err(NetErr::YAMLInvalidField(CONF_NAME_CGIMAP)),
        })
    }
    else {
        None
    };

    let docroot =
    if let Some(v) = map.remove(CONF_NAME_DOCROOT) {
        Some(match v {
            Value::String(s) => PathBuf::from(s),
            _ => return Err(NetErr::YAMLInvalidField(CONF_NAME_DOCROOT)),
        })
    }
    else {
        None
    };

    let default_file =
    if let Some(v) = map.remove(CONF_NAME_DEFAULT_FILE) {
        Some(match v {
            Value::String(s) => PathBuf::from(s),
            _ => return Err(NetErr::YAMLInvalidField(CONF_NAME_DEFAULT_FILE))
        })
    }
    else {
        None
    };

    let persisroot =
    if let Some(v) = map.remove(CONF_NAME_PERSISROOT) {
        Some(match v {
            Value::String(s) => PathBuf::from(s),
            _ => return Err(NetErr::YAMLInvalidField(CONF_NAME_PERSISROOT)),
        })
    }
    else {
        None
    };

    let listen_port =
    if let Some(v) = map.remove(CONF_NAME_LISTEN_PORT) {
        Some(if let Some(n) = v.as_u64() {
            n as u16
        }
        else {
            return Err(NetErr::YAMLInvalidField(CONF_NAME_LISTEN_PORT));
        })
    }
    else {
        None
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

    let timeout =
    if let Some(v) = map.remove(CONF_NAME_TIMEOUT) {
        Some(if let Some(n) = v.as_u64() {
            n
        }
        else {
            return Err(NetErr::YAMLInvalidField(CONF_NAME_TIMEOUT));
        })
    }
    else {
        None
    };


    Ok(ServConfOpt {
        cgiroot,
        cgimap: cgimapopt,
        docroot,
        default_file,
        persisroot,
        listen_port,
        // max_client,
        // init_client,
        timeout,
    })
}
