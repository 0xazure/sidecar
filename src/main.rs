use sidecar::Config;
use std::process;
use structopt::StructOpt;

fn main() {
    let config = Config::from_args();

    if let Err(e) = sidecar::run(config) {
        eprintln!("error: {}", e);
        process::exit(1);
    }
}
