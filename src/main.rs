use std::env;
use std::process;

extern crate futures;
extern crate hyper;
extern crate tokio_core;

use futures::{Future};
use hyper::{Client, Method, Request};
use tokio_core::reactor::Core;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });
    let token_info = if config.token.is_some() { "with token" } else { "without token" };


    let mut core = Core::new().unwrap();
    let client = Client::new(&core.handle());

    println!("Computing stats for GitHub repo {} {}" , config.repo, token_info);
    let url_as_str = "https://api.github.com/repos/".to_string() + &config.repo +
        "/events?access_token=" + &config.token.unwrap_or("".to_string()) + "&page=1";
    let uri = url_as_str.parse().unwrap();
    let mut req = Request::new(Method::Get, uri);
    req.headers_mut().set_raw("Accept", "application/vnd.github.v3+json");

    let request = client
        .request(req)
        .map(|res| {
            println!("HTTP status {}" , res.status());
        });

    // request is a Future, futures are lazy, so must explicitly run
    core.run(request).unwrap();

}

#[derive(Debug, PartialEq)]
struct Config {
    repo: String,
    token: Option<String>
}

impl Config {
    fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 2 {
            return Err("Not enough arguments, expecting at least 1 argument");
        }
        let repo = args[1].clone();
        let token = if args.len() == 3 {
            Some(args[2].clone())
        } else {
            None
        };
        Ok(Config { repo, token })
    }
}

#[cfg(test)]
mod test {

    use Config;

    #[test]
    fn parse_config_with_no_params() {
        let args: Vec<String> = vec!["".to_string()];
        assert_eq!(Config::new(&args), Err("Not enough arguments, expecting at least 1 argument"));
    }

    #[test]
    fn parse_config_with_repo_param() {
        let args: Vec<String> = vec!["".to_string(), "fakeRepo".to_string()];
        let repo = "fakeRepo".to_string();
        let token = None;
        assert_eq!(Config::new(&args), Ok(Config { repo, token }));
    }

    #[test]
    fn parse_config_with_repo_and_token_params() {
        let args: Vec<String> = vec!["".to_string(), "fakeRepo".to_string(), "fakeToken".to_string()];
        let repo = "fakeRepo".to_string();
        let token = Some("fakeToken".to_string());
        assert_eq!(Config::new(&args), Ok(Config { repo, token }));
    }
}