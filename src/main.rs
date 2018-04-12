use std::env;
use std::process;

extern crate pullpito;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = pullpito::Config::new(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    let token_info = if config.token.is_some() { "with token" } else { "without token" };
    println!("Computing stats for GitHub repo {} {}", config.repo, token_info);

    pullpito::github_events(config);
}
