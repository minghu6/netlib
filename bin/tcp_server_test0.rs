use clap::Parser;
// use libc::*;

/// Tcp Server Test0
#[derive(Parser)]
#[clap()]
struct Cli {
    #[clap(default_value_t = 8888)]
    port: u16
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    println!("listen port: {}", cli.port);

    Ok(())
}
