#![feature(box_syntax)]
#![feature(never_type)]

mod skbuff;
mod eth;

use clap::Parser;


/// Simple UDP/IP Network Protocol Stack
#[derive(Parser)]
#[clap(name = "SIP")]
struct Cli {
    // /// Hostname or IP
    // #[clap()]
    // dst: String,
}

fn main() {
    let _cli = Cli::parse();


}