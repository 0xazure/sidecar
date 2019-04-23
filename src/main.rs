use sidecar::Config;
use structopt::StructOpt;
use std::process;

fn main() {
    let config = Config::from_args();

    if let Err(e) = sidecar::run(config) {
        eprintln!("error: {}", e);
        process::exit(1);
    }
}
