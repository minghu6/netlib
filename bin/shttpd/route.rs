use std::{
    fmt::Write,
    fs::{create_dir_all, read_dir, read_to_string},
    iter::once_with,
    path::{Path, PathBuf},
    process::Command,
};

use chrono::{DateTime, Local};
use http::Method;
use log::info;
use mime::{TEXT_HTML, TEXT_HTML_UTF_8, TEXT_PLAIN, TEXT_PLAIN_UTF_8};
use qstring::QString;
use netlib::{
    application::http::parse_content_type,
    error::{NetErr, Result},
    or2s, s,
};

use crate::{
    conf::{CGIMap, CGIMapItem, ServConf},
    req::Req,
    resp::{Body, Resp},
};



pub struct RouteResolver {
    docroot: PathBuf,
    default_file: PathBuf,
    persisroot: PathBuf,
    cgimap: CGIMap,
}



impl RouteResolver {
    pub fn init(servconf: &ServConf) -> Result<Self> {
        // init persistent dir for each cgi
        for cgi in servconf.cgimap().items.iter() {
            create_dir_all(cgi.joined_by(&servconf.persisroot()))
                .or_else(|err| Err(NetErr::CreateDirAll(err)))?;
        }

        Ok(Self {
            docroot: servconf.docroot().clone(),
            default_file: servconf.default_file().clone(),
            persisroot: servconf.persisroot().clone(),
            cgimap: servconf.cgimap().clone(),
        })
    }

    pub fn resolve(&self, req: &Req) -> Resp {
        if req.method != Method::GET {
            return Resp::_405(String::new());
        }

        let paq = req.uri.path_and_query().unwrap();
        info!("url: {}", paq.path());

        if paq.path() == "/" {
            return loop {
                let index_path = self.docroot.join(&self.default_file);
                let body = match read_to_string(index_path) {
                    Ok(s) => Body::html(s),
                    Err(err) => break Resp::_500(err.to_string()),
                };

                break Resp::_200(body);
            };
        }

        if let Some(cgi) = self.cgimap.get(paq.path()) {
            info!("cgi map item: {cgi:?}");

            let q = QString::from(paq.query().unwrap_or_default());
            return match self.do_ipc_cgi(&cgi, q) {
                Ok(resp) => resp,
                Err(errmsg) => Resp::_500(errmsg),
            };
        }

        let docp = self.docroot.join(paq.path().strip_prefix("/").unwrap());
        info!("try access docp: {docp:?}");

        if docp.is_file() || docp.is_symlink() {
            return match read_to_string(docp) {
                Ok(s) => Resp::_200(Body::plain(s)),
                Err(err) => Resp::_500(err.to_string()),
            };
        }

        if docp.is_dir() {
            let mut s = String::new();
            return match read_dir_to_writeable(docp, &mut s) {
                Ok(_) => Resp::_200(Body::plain(s)),
                Err(errmsg) => Resp::_500(errmsg),
            };
        }

        Resp::_404(String::new())
    }


    pub fn do_ipc_cgi(
        &self,
        cgi: &CGIMapItem,
        q: QString,
    ) -> std::result::Result<Resp, String> {
        /* Generate env list from query */
        let envs = q
            .into_pairs()
            .into_iter()
            .map(|(k, v)| (format!("SHTTPD_Q_{}", k.to_uppercase()), v))
            .chain(once_with(|| {
                (
                    s!("SHTTPD_PERSIS_ROOT"),
                    s!(self.persisroot.to_string_lossy()),
                )
            }))
            .chain(once_with(|| {
                (
                    s!("SHTTPD_PERSIS_DIR"),
                    s!(cgi.joined_by(&self.persisroot).to_string_lossy()),
                )
            }));

        let output =
            or2s!(Command::new(cgi.joined_by(&self.cgimap.root).as_os_str())
                .current_dir(or2s!(std::env::current_dir())?)
                .envs(envs)
                .output())?;

        if !output.stderr.is_empty() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        let out_decoded = String::from_utf8_lossy(&output.stdout);
        let lf_end = if let Some(lf_end) = out_decoded.find("\n") {
            lf_end
        }
        else {
            return Err(s!("There are no first line"));
        };

        info!("cgi-content-type: {}", &out_decoded[0..lf_end]);
        let content_type = parse_content_type(&out_decoded[0..lf_end])
            .or_else(|_| Err("CGI Invalid Content-Type"))?;
        let remains = &out_decoded[lf_end + 1..];

        let body = if content_type == TEXT_PLAIN
            || content_type == TEXT_PLAIN_UTF_8
        {
            Body::plain(remains.to_owned())
        }
        else if content_type == TEXT_HTML || content_type == TEXT_HTML_UTF_8
        {
            Body::html(remains.to_owned())
        }
        else {
            return Err(format!("Unsupported cgi mime type: {content_type}"));
        };


        Ok(Resp::_200(body))
    }
}




fn read_dir_to_writeable<P: AsRef<Path>, W: Write>(
    p: P,
    w: &mut W,
) -> std::result::Result<(), String> {
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
