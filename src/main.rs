use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });
    println!("Computing stats for GitHub repo {}", config.repo);
}

struct Config {
    repo: String,
    token: Option<String>
}

impl Config {
    fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 1 {
            return Err("Not enough arguments, expecting at least 1 argument");
        }
        let repo = args[1].clone();
        let token = if args.len() == 2 {
            Some(args[1].clone())
        } else {
            None
        };
        Ok(Config { repo, token })
    }
}