extern crate env_logger;
#[macro_use]
extern crate log;
extern crate pullpito;

use std::env;
use std::process;

fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let config = pullpito::Config::new(&args).unwrap_or_else(|err| {
        error!("Problem parsing arguments: {}", err);
        process::exit(1);
    });
    info!(
        "Computing stats for GitHub repos '{:?}' (with token: {})",
        config.repos,
        config.token.is_some()
    );
    pullpito::github_events(config);
}
