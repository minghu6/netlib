//! Http 0.9, 1.0, 1.1
//!
use std::{str::FromStr, convert::Infallible};

use cookie::Cookie;
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


