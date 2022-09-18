use std::str::FromStr;

use http::{
    header::{HeaderName, ACCEPT, COOKIE},
    uri::{PathAndQuery, Scheme},
    HeaderValue, Method, Uri, Version, HeaderMap,
};
use itertools::Itertools;
use netlib::{application::http::{HeaderAccept, HeaderCookie}, error::*, s};

use crate::resp::Body;


////////////////////////////////////////////////////////////////////////////////
//// Structure

/// Http 0.9-1.1 Request
#[derive(Debug)]
pub struct Req {
    pub version: Version,
    pub method: Method,
    pub uri: Uri,
    pub accept: HeaderAccept,
    pub cookie: HeaderCookie,
    pub body: Body,
}


////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl Req {
    pub fn parse_from_bytes(
        bytes: &[u8],
        filln: usize,
    ) -> std::result::Result<Req, HttpKind> {
        // let plain = String::from_utf8_lossy(&bytes[..filledn]);
        if filln < 3 * 2 {
            return Err(HttpKind::TooShort(filln));
        }

        // bytes[filledn] = 0, on guard
        let mut i = 0;
        loop {
            if bytes[i] == '\r' as u8 && bytes[i + 1] == '\n' as u8 {
                break Ok(());
            }
            if i >= filln - 2 {
                break Err(HttpKind::InvalidReqLn);
            }

            i += 1;
        }?;

        // A request line containing only the path name is accepted by servers
        // to maintain compatibility with HTTP clients before the HTTP/1.0
        // specification in RFC 1945.
        let reqln = &bytes[..i];
        let (mb, urlb, verb) = reqln
            .split(|c| *c == ' ' as u8)
            .next_tuple()
            .ok_or(HttpKind::InvalidReqLn)?;

        let method = Method::from_bytes(mb)
            .or_else(|_err| Err(HttpKind::InvalidReqLn))?;

        let pqs = String::from_utf8_lossy(urlb);
        let pq = PathAndQuery::from_str(&pqs)
            .or_else(|_err| Err(HttpKind::InvalidReqLn))?;
        if !verb.starts_with(b"HTTP/") {
            return Err(HttpKind::InvalidReqLn);
        }
        let uri = Uri::builder()
            .scheme(Scheme::HTTP)
            .authority("localhost")
            .path_and_query(pq)
            .build()
            .or_else(|_err| Err(HttpKind::InvalidReqLn))?;
        let version = match &verb[5..8] {
            b"0.9" => Version::HTTP_09,
            b"1.0" => Version::HTTP_10,
            b"1.1" => Version::HTTP_11,
            _ => {
                return Err(HttpKind::InvalidReqLn);
            }
        };


        /* Let's parsing headers now */
        i += 2;

        let mut hdrs = HeaderMap::new();
        /* DETECT AN EMPTYLINE */
        if bytes[i] == '\r' as u8 && bytes[i + 1] == '\n' as u8 {
            if version != Version::HTTP_09 {
                return Err(HttpKind::HeaderNotFoundForVersion(format!(
                    "{:#?}",
                    version
                )));
            }
        }
        else {
            // zero or more request header fields (at least 1
            // or more headers in case of HTTP/1.1)
            // let mut headers = HeaderMap::new();
            let mut hdrlns = vec![];
            let mut start = i;

            loop {
                if bytes[i] == '\r' as u8 && bytes[i + 1] == '\n' as u8 {
                    if start == i {
                        // EMPTY LINE found!
                        break Ok(());
                    }
                    // else push and start next header scan
                    hdrlns.push(&bytes[start..i]);
                    start = i + 2;
                    if start == filln {
                        break Ok(());
                    }
                }

                if i >= filln - 2 {
                    break Err(HttpKind::InvalidHeader(s!(
                        "Unexpected header item END"
                    )));
                }

                i += 1;
            }?;

            for ln in hdrlns {
                let (pos_colon, _) = ln
                    .into_iter()
                    .find_position(|c| **c == ':' as u8)
                    .ok_or(HttpKind::InvalidHeader(format!(
                        "There is no colon sign in header line:\n{ln:#?}"
                    )))?;

                let name =
                    HeaderName::from_bytes(&ln[..pos_colon]).or_else(|err| {
                        Err(HttpKind::InvalidHeader(format!("{}", err)))
                    })?;

                let value = HeaderValue::from_bytes(&ln[pos_colon + 1..])
                    .or_else(|err| {
                        Err(HttpKind::InvalidHeader(format!("{}", err)))
                    })?;

                hdrs.insert(name, value);
            }
        }

        /* Handle split header */
        /* Accept */
        let accept = if let Some(val) = hdrs.get(ACCEPT) {
            match val.to_str().unwrap().parse() {
                Ok(res) => res,
                Err(_) => Default::default(),
            }
        }
        else {
            Default::default()
        };

        /* Cookie */
        let cookie = if let Some(val) = hdrs.get(COOKIE) {
            match val.to_str().unwrap().parse() {
                Ok(res) => res,
                Err(_) => Default::default(),
            }
        }
        else {
            Default::default()
        };


        Ok(Req {
            version,
            method,
            uri,
            accept,
            cookie,
            body: Body::Empty,
        })
    }

}
