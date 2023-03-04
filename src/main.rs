use siphon;
use std::env;
use std::process;

fn main() {
    let args = env::args();

    let config = siphon::Config::build(args).unwrap_or_else(|err| {
        eprintln!("Problem processing arguments: {err}");
        process::exit(1);
    });

    siphon::run(config).unwrap_or_else(|err| {
        eprintln!("Application error: {err}");
        process::exit(1);
    });
}
