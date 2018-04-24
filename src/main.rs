#[macro_use]
extern crate log;

use std::env;
use std::process;

extern crate env_logger;
extern crate pullpito;

fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let config = pullpito::Config::new(&args).unwrap_or_else(|err| {
        error!("Problem parsing arguments: {}", err);
        process::exit(1);
    });
    info!(
        "Computing stats for GitHub repo '{}' (with token: {})",
        config.repo,
        config.token.is_some()
    );
    pullpito::github_events(config);
}
