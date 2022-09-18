use std::{
    fmt::Write,
    fs::{read_dir, read_to_string},
    path::{Path, PathBuf},
};

use chrono::{DateTime, Local};
use http::Method;
use log::info;
use netlib::or2s;
use qstring::QString;

use crate::{
    cgi::do_ipc_cgi,
    conf::CGIMap,
    req::Req,
    resp::{Body, Resp},
};



pub struct RouteResolver {
    pub docroot: PathBuf,
    pub default_file: PathBuf,
    pub cgimap: CGIMap,
}



impl RouteResolver {
    pub fn resolve(&self, req: &Req) -> Resp {
        if req.method != Method::GET {
            return Resp::_405("");
        }

        let paq = req.uri.path_and_query().unwrap();
        info!("url: {}", paq.path());

        if paq.path() == "/" {
            return loop {
                let index_path = self.docroot.join(&self.default_file);
                let body = match read_to_string(index_path) {
                    Ok(s) => Body::Html(s),
                    Err(err) => break Resp::_500(&err.to_string()),
                };

                break Resp::_200(body);
            };
        }

        if let Some(cgiitem) = self.cgimap.get(paq.path()) {
            let cgip = self.cgimap.root.join(&cgiitem.cgi);
            let q = QString::from(paq.query().unwrap_or_default());
            return do_ipc_cgi(&cgip, &q);
        }

        let docp = self.docroot.join(paq.path().strip_prefix("/").unwrap());
        info!("try access docp: {docp:?}");

        if docp.is_file() || docp.is_symlink() {
            return match read_to_string(docp) {
                Ok(s) => Resp::_200(Body::Plain(s)),
                Err(err) => Resp::_500(&err.to_string()),
            };
        }

        if docp.is_dir() {
            let mut s = String::new();
            return match read_dir_to_writeable(docp, &mut s) {
                Ok(_) => Resp::_200(Body::Plain(s)),
                Err(errmsg) => Resp::_500(&errmsg),
            }
        }

        Resp::_404("")
    }
}




fn read_dir_to_writeable<P: AsRef<Path>, W: Write>(
    p: P,
    w: &mut W,
) -> Result<(), String> {
    for dir_ent_res in or2s!(read_dir(p))? {
        let dir_ent = or2s!(dir_ent_res)?;

        let name = dir_ent.file_name();
        let meta = or2s!(dir_ent.metadata())?;

        let mut ft = String::new();

        if meta.is_file() {
            ft.push('-')
        }
        if meta.is_dir() {
            ft.push('d')
        }
        if meta.is_symlink() {
            ft.push('s')
        }

        let dt = DateTime::<Local>::from(
            if let Ok(modified_time) = meta.modified() {
                modified_time
            }
            else if let Ok(created_time) = meta.created() {
                created_time
            }
            else {
                return Err(format!(
                    "{:?} get modify/create meta field failed",
                    dir_ent.path()
                ));
            },
        );

        writeln!(w, "{ft:2} {dt} {name:<20?}").unwrap();
    }

    Ok(())
}




#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[test]
    fn test_path_join() {
        let a = PathBuf::from("/a");
        let b = "/b/b1/b2.bb";

        println!("{:?}", a.join(b));

        let a = PathBuf::from("/a/");
        println!("{:?}", a.join(b));

        let b = "b/b1/b2.bb";
        println!("{:?}", a.join(b));
    }
}
