use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = parse_config(&args);
    println!("Computing stats for GitHub repo {}", config.repo);
}

struct Config {
    repo: String,
    token: Option<String>
}

fn parse_config(args: &[String]) -> Config {
    if args.len() < 1 {
        panic!("Not enough arguments, expecting at least 1 argument");
    }
    let repo = args[1].clone();
    let token = if args.len() == 2 {
        Some(args[1].clone())
    } else {
        None
    };
    Config { repo, token }
}