extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate serde_json;
extern crate tokio_core;

#[macro_use] extern crate serde_derive;

use futures::{Future, Stream};
use hyper::{Client, Method, Request};
use hyper_tls::HttpsConnector;
use serde_json::Error;
use tokio_core::reactor::Core;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Config { pub repo: String, pub token: Option<String> }

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
            res.body().concat2().and_then(move |_body| {
                println!("Body found");
                let _events = raw_github_events("".to_string());
                Ok("")
            })
        });
    core.run(work).unwrap();
}

#[derive(Debug, PartialEq)]
struct GithubEvent { author: String, opened_pr: u8 }

#[derive(Debug, Deserialize, PartialEq)]
struct RawEvent { actor: Actor }

#[derive(Debug, Deserialize, PartialEq)]
struct Actor { login: String }

fn raw_github_events(json: String) -> Result<Vec<RawEvent>, Error> {
    return serde_json::from_str::<Vec<RawEvent>>(&json);
}

#[cfg(test)]
mod test {
    use Config;

    #[test]
    fn parse_config_with_no_params() {
        assert_eq!(
            Config::new(&vec!["".to_string()]),
            Err("Not enough arguments, expecting at least 1 argument"));
    }

    #[test]
    fn parse_config_with_repo_param() {
        assert_eq!(
            Config::new(&vec!["".to_string(), "fakeRepo".to_string()]),
            Ok(Config { repo: "fakeRepo".to_string(), token: None }));
    }

    #[test]
    fn parse_config_with_repo_and_token_params() {
        assert_eq!(
            Config::new(&vec!["".to_string(), "fakeRepo".to_string(), "fakeToken".to_string()]),
            Ok(Config { repo: "fakeRepo".to_string(), token: Some("fakeToken".to_string()) }));
    }

    use {raw_github_events, RawEvent, Actor};

    #[test]
    fn parse_github_events() {
        assert_eq!(
            raw_github_events(include_str!("../test/github_events.json").to_string()).unwrap()[0],
            RawEvent { actor: Actor { login: "alice".to_string() } });
    }
}