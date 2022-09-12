#![feature(arbitrary_enum_discriminant)]
#![feature(exclusive_range_pattern)]
#![feature(box_syntax)]
#![feature(type_alias_impl_trait)]

pub mod network;
pub mod transport;
pub mod error;
pub mod aux;
pub mod data;
pub mod err;
pub mod imp;


#[cfg(test)]
mod tests {

}

