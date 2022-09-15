#![allow(unused_imports)]

use std::{
    fmt::Write,
    io::Read,
    net::TcpStream,
    str::FromStr, time::Duration,
};

use bytes::{Bytes, BytesMut, BufMut};
use http::{
    header::HeaderName,
    request,
    uri::{PathAndQuery, Scheme}, HeaderValue, Method, Request, Uri, Version,
};
use itertools::Itertools;
use log::info;
use netlib::{error::*, s};

use crate::conf::ServConf;


pub async fn do_work(conf: ServConf, stream: TcpStream) {
    match _do_work(conf, stream) {
        Ok(_) => (),
        Err(err) => eprintln!("#{err}"),
    }
}


const REQ_BUF_LEN: usize = 1024 * 2;

fn _do_work(conf: ServConf, mut stream: TcpStream) -> Result<()> {
    stream.set_read_timeout(Some(Duration::from_millis(*conf.timeout())))
    .or_else(|err| Err(NetErr::SetStreamOpt(err)))?;

    let mut bytes_raw = [0u8; REQ_BUF_LEN];

    let filln = stream
        .read(&mut bytes_raw)
        .or_else(|err| Err(NetErr::Read(err)))?;

    // on guard
    let bytes = BytesMut::from(&bytes_raw[..]);

    let req = parse_req(bytes.freeze(), filln)
        .or_else(|kind| Err(NetErr::HttpBadReq(kind)))?;


    info!("Incomming request: \n{:#?}\n", req);

    // In HTTP/1.0, as stated in RFC 1945, the TCP/IP connection should always
    // be closed by server after a response has been sent.
    stream
        .shutdown(std::net::Shutdown::Write)
        .or_else(|err| Err(NetErr::StreamShutDown(err)))?;

    Ok(())
}


/// Http 1.1
fn parse_req(
    bytes: Bytes,
    filln: usize,
) -> std::result::Result<Request<()>, HttpKind> {
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

    let m =
        Method::from_bytes(mb).or_else(|_err| Err(HttpKind::InvalidReqLn))?;

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
    let ver = match &verb[5..8] {
        b"0.9" => Version::HTTP_09,
        b"1.0" => Version::HTTP_10,
        b"1.1" => Version::HTTP_11,
        _ => {
            return Err(HttpKind::InvalidReqLn);
        }
    };


    /* DETECT AN EMPTYLINE */
    let mut req_builder =
        request::Builder::new().method(m).uri(uri).version(ver);

    i += 2;
    if bytes[i] == '\r' as u8 && bytes[i + 1] == '\n' as u8 {
        if ver == Version::HTTP_09 {
            // Parse Body
            let req = req_builder
                .body(())
                .or_else(|err| Err(HttpKind::Bug(format!("{:#?}", err))))?;

            return Ok(req);
        }
        else {
            return Err(HttpKind::HeaderNotFoundForVersion(format!(
                "{:#?}",
                ver
            )));
        }
    }

    // zero or more request header fields (at least 1
    // or more headers in case of HTTP/1.1)
    // let mut headers = HeaderMap::new();
    i += 2; // CRLF exceeded
    let mut start = i;

    let mut hdrlns = vec![];
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
            break Err(HttpKind::InvalidHeader(s!("Unexpected header item END")));
        }

        i += 1;
    }?;

    for ln in hdrlns {
        let (pos_colon, _) = ln
            .into_iter()
            .find_position(|c| **c == ':' as u8)
            .ok_or(HttpKind::InvalidHeader(format!("There is no colon sign in header line:\n{ln:#?}")))?;

        let name = HeaderName::from_bytes(&ln[..pos_colon])
            .or_else(|err| Err(HttpKind::InvalidHeader(format!("{}", err))))?;

        let value = HeaderValue::from_bytes(&ln[pos_colon + 1..])
            .or_else(|err| Err(HttpKind::InvalidHeader(format!("{}", err))))?;

        req_builder = req_builder.header(name, value)
    }

    let req = req_builder
        .body(())
        .or_else(|err| Err(HttpKind::InvalidHeader(format!("{}", err))))?;

    Ok(req)
}



#[cfg(test)]
mod tests {
    use std::fmt::Write;

    use bytes::{ BytesMut, BufMut};


    #[test]
    fn test_bytes() {
        let mut buf = BytesMut::with_capacity(1024);
        buf.put(&b"hello world"[..]);
        // buf.put(&b"123456"[..]);
        buf.write_str("123456");

        println!("{:#?}", buf);
    }
}
