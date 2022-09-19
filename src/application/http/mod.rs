//! Http 0.9, 1.0, 1.1
//!
use std::{str::FromStr, convert::Infallible, fmt::Display};

use cookie::Cookie;
use http::header::CONTENT_TYPE;
use mime::{ Mime, self};
use lazy_static::lazy_static;
use regex::Regex;

use crate::error::HttpKind;

////////////////////////////////////////////////////////////////////////////////
//// Structure

#[derive(Debug, Default)]
pub struct HeaderAccept {
    items: Vec<AcceptItem>
}


#[derive(Debug)]
pub struct AcceptItem {
    names: Vec<Mime>,
    pub q: Option<f32>,
}


#[derive(Debug, Default)]
pub struct HeaderAcceptEncoding {
    pub items: Vec<AcceptEncoding>
}


#[derive(Debug, PartialEq, Eq)]
pub enum AcceptEncoding {
    /// using the Lempel-Ziv coding (LZ77), with a 32-bit CRC.
    ///
    /// This is the original format of the UNIX gzip program
    Gzip,

    /// using the Lempel-Ziv-Welch (LZW) algorithm
    Compress,

    /// Using the zlib structure with the deflate compression algorithm
    Deflate,

    /// Using the Brotli algorithm Non Standard
    Br
}


#[derive(Debug, Default)]
pub struct HeaderCookie {
    items: Vec<Cookie<'static>>
}





////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl AcceptItem {
    pub fn contains(&self, mime: &Mime) -> bool {
        self.names
        .iter()
        .find(|name| *name == mime)
        .is_some()
    }
}


impl HeaderAccept {
    pub fn accept(&self, m: &Mime) -> bool {
        if self.get_by_mime(&m).is_some()
        || *m != mime::STAR_STAR && self.get_by_mime(&mime::STAR_STAR).is_some() {
            return true;
        }

        if m.type_() == mime::TEXT {
            self.get_by_mime(&mime::TEXT_STAR).is_some()
        }
        else {
            false
        }
    }

    pub fn get_by_mime(&self, m: &Mime) -> Option<&AcceptItem> {
        self.items
        .iter()
        .find(|x| {
            if (*x).contains(m) {
                true
            }
            else {
                false
            }
        })
    }

}


/// Lossy match, invalid field would be ignored.
impl FromStr for HeaderAccept {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut items = vec![];
        for item_s in s.trim_start().split(";") {
            let mut names = vec![];
            let mut q = None;
            for mopts in item_s.split(",") {
                lazy_static! {
                    static ref QNUM: Regex = Regex::new("q=((1.0)|(1)|(0.[1-9]))").unwrap();
                }

                if let Some(cap) = QNUM.captures(mopts) {
                    if let Some(v) = cap.get(1) {
                        q = Some(v.as_str().parse().unwrap());
                        continue;
                    }
                }

                if let Ok(mime) = Mime::from_str(mopts) {
                    names.push(mime);
                }
            }

            items.push(AcceptItem {
                names,
                q,
            })
        }

        Ok(HeaderAccept {
            items
        })
    }
}


impl HeaderCookie {
    pub fn get(&self, name: &str) -> Option<&Cookie> {
        self.items
        .iter()
        .find(|cookie| cookie.name() == name)
    }

    // pub fn get_mut(&'static mut self, name: &str) -> Option<&mut Cookie> {
    //     self.items
    //     .iter_mut()
    //     .find(|cookie| (**cookie).name() == name)
    // }
}


/// Lossy match, invalid field would be ignored.
impl FromStr for HeaderCookie {
    type Err = HttpKind;

    fn from_str(s: &str) -> Result<Self, Self::Err> {

        let mut items = vec![];
        for item_s in s.trim_start().split(";") {
            if let Ok(cookie) = Cookie::from_str(item_s.trim_start()) {
                items.push(cookie);
            }
        }

        Ok(Self {
            items
        })
    }
}


impl FromStr for HeaderAcceptEncoding {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut items = vec![];
        for item_s in s.trim_start().split(",") {
            if let Ok(encoding) = AcceptEncoding::from_str(item_s.trim()) {
                items.push(encoding);
            }
        }

        Ok(Self {
            items
        })
    }
}


impl HeaderAcceptEncoding {
    pub fn contains(&self, encoding: &AcceptEncoding) -> bool {
        self
        .items
        .iter()
        .find(|x| *x == encoding)
        .is_some()
    }
}


impl FromStr for AcceptEncoding {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "gzip" => Self::Gzip,
            "compress" => Self::Compress,
            "deflate" => Self::Deflate,
            "br" => Self::Br,
            _ => return Err(())
        })
    }
}


impl Display for AcceptEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}",
            match self {
                Self::Gzip => "gzip",
                Self::Compress => "compress",
                Self::Deflate => "deflate",
                Self::Br => "br",
            }
        )
    }
}



////////////////////////////////////////////////////////////////////////////////
//// Function


pub fn parse_content_type(s: &str) -> Result<Mime, ()> {
    let s = s.trim().to_lowercase();
    let prefix = format!("{CONTENT_TYPE}: ");

    if !s.starts_with(&prefix) {
        return Err(());
    }

    Mime::from_str(&s[prefix.len()..])
    .or_else(|_| Err(()))

}


#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use mime::Mime;


    #[test]
    fn test_mime_from() {
        let s = "text/plain; charset=UTF-8";

        let m = Mime::from_str(&s).unwrap();

        println!("m: {m}");
    }
}