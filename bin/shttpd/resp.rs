use std::fmt::Write;

use bytes::BytesMut;
use http::{
    header::{CONTENT_TYPE, DATE, SERVER, CONNECTION, CONTENT_LENGTH},
    StatusCode, Version,
};
use mime::{Mime, TEXT_PLAIN_UTF_8, TEXT_HTML_UTF_8};
use netlib::s;


pub const SERVER_NAME: &str = "Shttpd-minghu6 (Linux)";

////////////////////////////////////////////////////////////////////////////////
//// Structure

/// Http 0.9-1.1 Response
#[derive(Debug)]
pub struct Resp {
    version: Version,
    status: StatusCode,
    date: String,
    // content_length: u64,, replacing with body.len()
    server: String,
    is_close: bool,

    body: Body,
}


#[derive(Debug)]
pub struct Body {
    pub content_type: Mime,
    pub body: String
}



////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl Body {
    pub fn len(&self) -> usize {
        self.body.len()
    }

    pub fn plain(raw: String) -> Self {
        Self {
            content_type: TEXT_PLAIN_UTF_8,
            body: raw,
        }
    }

    pub fn html(raw: String) -> Self {
        Self {
            content_type: TEXT_HTML_UTF_8,
            body: raw,
        }
    }

    pub fn empty() -> Self {
        Self {
            content_type: TEXT_PLAIN_UTF_8,
            body: String::new(),
        }
    }

}



impl Resp {
    pub fn _404(msg: String) -> Self {
        Self {
            version: Version::HTTP_11,
            status: StatusCode::NOT_FOUND,
            date: chrono::offset::Local::now().to_rfc2822(),
            server: s!(SERVER_NAME),
            is_close: true,
            body: Body::plain(msg),
        }
    }

    pub fn _405(msg: String) -> Self {
        Self {
            version: Version::HTTP_11,
            status: StatusCode::METHOD_NOT_ALLOWED,
            date: chrono::offset::Local::now().to_rfc2822(),
            server: s!(SERVER_NAME),
            is_close: true,
            body: Body::plain(msg),
        }
    }

    pub fn _500(msg: String) -> Self {
        Self {
            version: Version::HTTP_11,
            status: StatusCode::INTERNAL_SERVER_ERROR,
            date: chrono::offset::Local::now().to_rfc2822(),
            server: s!(SERVER_NAME),
            is_close: true,
            body: Body::plain(msg),
        }
    }

    pub fn _200(body: Body) -> Self {
        Self {
            version: Version::HTTP_11,
            status: StatusCode::OK,
            date: chrono::offset::Local::now().to_rfc2822(),
            server: s!(SERVER_NAME),
            is_close: false,
            body,
        }
    }

    pub fn into_bytes(self, bytes: &mut BytesMut) {
        let Self {
                version,
                status,
                date,
                server,
                is_close,
                body,
            } = self;

        let Body { content_type, body } = body;

        /* Write status line */
        write!(bytes, "{version:?} {status}\r\n").unwrap();
        write!(bytes, "{DATE}: {date}\r\n").unwrap();
        write!(bytes, "{SERVER}: {server}\r\n").unwrap();
        write!(bytes, "{CONTENT_TYPE}: {content_type}\r\n").unwrap();
        write!(bytes, "{CONTENT_LENGTH}: {}\r\n", body.len()).unwrap();

        if is_close {
            write!(bytes, "{CONNECTION}: close\r\n").unwrap();
        }

        write!(bytes, "\r\n").unwrap();
        write!(bytes, "{body}").unwrap();
    }
}




#[cfg(test)]
mod tests {
    use http::Version;


    #[test]
    fn test_write_response() {
        println!("{:?}", Version::HTTP_11);
    }
}
