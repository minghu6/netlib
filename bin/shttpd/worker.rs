#![allow(unused_imports)]

use std::{
    fs::read_to_string,
    io::{Read, Write},
    net::TcpStream,
    str::FromStr,
    time::Duration, sync::Arc,
};

use http::{
    header::{HeaderName, ACCEPT, ACCEPT_RANGES, COOKIE},
    request,
    uri::{PathAndQuery, Scheme},
    HeaderValue, Method, Request, Uri, Version,
};
use itertools::Itertools;
use lazy_static::lazy_static;
use log::info;
use netlib::{rs_error::*, s, application::http::AcceptEncoding};

use crate::req::*;
use crate::{
    conf::ServConf,
    resp::{Body, Resp},
    route::RouteResolver,
    GloablContext,
};


const REQ_BUF_LEN: usize = 1024 * 2;



pub async fn do_work(ctx:Arc<GloablContext>, stream: TcpStream) {
    match _do_work(ctx, stream) {
        Ok(_) => (),
        Err(err) => eprintln!("#{err}"),
    }
}


fn _do_work(ctx: Arc<GloablContext>, mut stream: TcpStream) -> Result<()> {
    stream
        .set_read_timeout(Some(Duration::from_millis(*ctx.servconf.timeout())))
        .or_else(|err| Err(NetErr::SetStreamOpt(err)))?;

    stream
        .set_write_timeout(Some(Duration::from_millis(
            *ctx.servconf.timeout(),
        )))
        .or_else(|err| Err(NetErr::SetStreamOpt(err)))?;

    let mut bytes_raw = [0u8; REQ_BUF_LEN];

    let filln = stream
        .read(&mut bytes_raw)
        .or_else(|err| Err(NetErr::Read(err)))?;

    // on guard
    let req = Req::parse_from_bytes(&bytes_raw, filln)
        .or_else(|kind| Err(NetErr::HttpBadReq(kind)))?;

    info!("Incomming request: \n{:#?}\n", req);

    let mut bytes_resp = Vec::new();
    let resp = ctx.resolver.resolve(&req);

    let encoding =
    if req.accept_encoding.contains(&AcceptEncoding::Gzip) {
        Some(AcceptEncoding::Gzip)
    }
    else {
        None
    };

    resp.into_bytes(&mut bytes_resp, encoding);

    stream
        .write(bytes_resp.as_ref())
        .or_else(|err| Err(NetErr::Write(err)))?;

    // In HTTP/1.0, as stated in RFC 1945, the TCP/IP connection should always
    // be closed by server after a response has been sent.
    stream
        .shutdown(std::net::Shutdown::Write)
        .or_else(|err| Err(NetErr::StreamShutDown(err)))?;

    Ok(())
}
