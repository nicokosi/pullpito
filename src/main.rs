use env_logger;
use log::info;
use pullpito;

use std::env;

fn main() {
    env_logger::init();
    let config = pullpito::config_from_args(env::args_os().collect());
    info!(
        "Computing stats for GitHub repos '{:?}' (with token: {})",
        config.repos,
        config.token.is_some()
    );
    pullpito::github_events(config);
}
