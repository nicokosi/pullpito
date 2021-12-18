use std::env;

fn main() {
    pullpito::log_github_events(env::args_os().collect());
}
