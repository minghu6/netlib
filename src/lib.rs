#![feature(arbitrary_enum_discriminant)]
#![feature(exclusive_range_pattern)]
#![feature(box_syntax)]
#![feature(type_alias_impl_trait)]

pub mod datalink;
pub mod network;
pub mod transport;
pub mod application;
pub mod rs_error;
pub mod c_error;
pub mod aux;
pub mod data;
pub mod view;
pub mod dev;


pub use rs_error::{ Result, NetErr };
pub type RawResult<T, E> = std::result::Result<T, E>;


#[cfg(test)]
mod tests {

}

