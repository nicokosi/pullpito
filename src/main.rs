use std::env;
use std::process;

extern crate pullpito;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = pullpito::Config::new(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });
    pullpito::github_events(config);
}
