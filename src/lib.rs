extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;

use futures::Future;
use hyper::{Client, Method, Request};
use hyper_tls::HttpsConnector;
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
    println!("GitHub request: {:?}", &req);

    let request = client
        .request(req)
        .map(|res| {
            println!("HTTP status {}", res.status());
        });

    core.run(request).unwrap();
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
