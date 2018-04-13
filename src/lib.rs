extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate serde_json;
extern crate tokio_core;

use futures::{Future, Stream};
use hyper::{Client, Method, Request};
use hyper_tls::HttpsConnector;
use serde_json::Value;
use std::io;
use tokio_core::reactor::Core;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub repo: String,
    pub token: Option<String>,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
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

pub fn github_events(config: Config) {
    let mut core = Core::new().unwrap();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &core.handle()).unwrap())
        .build(&core.handle());

    let url = "https://api.github.com/repos/".to_string() + &config.repo + "/events?access_token="
        + &config.token.unwrap_or("".to_string()) + "&page=1";
    let mut req = Request::new(Method::Get, url.parse().unwrap());
    req.headers_mut().set_raw("Accept", "application/vnd.github.v3+json");
    req.headers_mut().set_raw("Host", "api.github.com");
    req.headers_mut().set_raw("User-Agent", "pullpito/0.1.0");

    let work = client
        .request(req)
        .and_then(|res| {
            println!("HTTP status {}", res.status());
            res.body().concat2().and_then(move |body| {
                println!("Body found");
                let v: Value = serde_json::from_slice(&body).map_err(|e| {
                    io::Error::new(io::ErrorKind::Other, e)
                })?;
                println!("current IP address is {}", v["created_at"]);
                Ok(())
            })
        });
    core.run(work).unwrap();
}

#[derive(Debug, PartialEq)]
struct GithubEvent {
    pub author: String,
    pub opened_pr: u8
}

fn raw_github_events(json: String) -> Result<Vec<GithubEvent>, String> {
    let v: Value = serde_json::from_str(json.as_ref())
        .map_err(|e| {
            io::Error::new(io::ErrorKind::Other, e)
        }).unwrap();
    println!("current IP address is {}", v["actor"]);
    return Ok(Vec::new());
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

    use raw_github_events;
    use GithubEvent;

    #[test]
    fn parse_github_events() {
        let events = include_str!("../test/github_events.json");
        assert_eq!(
            raw_github_events(events.to_string()),
            Ok(vec![
                GithubEvent { author: "alice".to_string(), opened_pr: 1 },
                GithubEvent { author: "bob".to_string(), opened_pr: 2 },
                GithubEvent { author: "carol".to_string(), opened_pr: 1 }
            ]));
    }

}