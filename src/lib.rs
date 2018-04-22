extern crate chrono;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate serde_json;
extern crate tokio_core;

#[macro_use]
extern crate serde_derive;

use futures::{Future, Stream};
use hyper::{Client, Method, Request};
use hyper_tls::HttpsConnector;
use serde_json::Error;
use tokio_core::reactor::Core;
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
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

use std::str;

pub fn github_events(config: Config) {
    let mut core = Core::new().unwrap();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &core.handle()).unwrap())
        .build(&core.handle());

    let url = "https://api.github.com/repos/".to_string() + &config.repo + "/events?access_token="
        + &config.token.unwrap_or("".to_string()) + "&page=1";
    let mut req = Request::new(Method::Get, url.parse().unwrap());
    req.headers_mut()
        .set_raw("Accept", "application/vnd.github.v3+json");
    req.headers_mut().set_raw("Host", "api.github.com");
    req.headers_mut().set_raw("User-Agent", "pullpito/0.1.0");

    println!("Request: {:?}", &req);
    let work = client.request(req).and_then(|res| {
        res.body().concat2().and_then(move |body| {
            let raw_events_as_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
            let raw_events = raw_github_events(raw_events_as_json.to_string());
            raw_events.unwrap().iter()
                .filter(| e | e.event_type == Type::PullRequestEvent && e.payload.action == Some(Action::created))
                .collect::<Vec<RawEvent>>()
                .size();
            Ok(())
        })
    });

    core.run(work).unwrap();
}

#[derive(Debug, Deserialize, PartialEq)]
struct RawEvent {
    actor: Actor,
    payload: Payload,
    #[serde(rename = "type")]
    event_type: Type,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Actor {
    login: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Payload {
    action: Option<Action>,
}

#[derive(Debug, Deserialize, PartialEq)]
enum Action {
    #[allow(non_camel_case_types)]
    created,
    #[allow(non_camel_case_types)]
    closed,
    #[allow(non_camel_case_types)]
    edited,
    #[allow(non_camel_case_types)]
    opened,
    #[allow(non_camel_case_types)]
    started,
}

#[derive(Debug, Deserialize, PartialEq)]
enum Type {
    CommitCommentEvent,
    CreateEvent,
    DeleteEvent,
    DeploymentEvent,
    DeploymentStatusEvent,
    DownloadEvent,
    FollowEvent,
    ForkEvent,
    ForkApplyEvent,
    GistEvent,
    GollumEvent,
    InstallationEvent,
    InstallationRepositoriesEvent,
    IssueCommentEvent,
    IssuesEvent,
    LabelEvent,
    MarketplacePurchaseEvent,
    MemberEvent,
    MembershipEvent,
    MilestoneEvent,
    OrganizationEvent,
    OrgBlockEvent,
    PageBuildEvent,
    ProjectCardEvent,
    ProjectColumnEvent,
    ProjectEvent,
    PublicEvent,
    PullRequestEvent,
    PullRequestReviewEvent,
    PullRequestReviewCommentEvent,
    PushEvent,
    ReleaseEvent,
    RepositoryEvent,
    StatusEvent,
    TeamEvent,
    TeamAddEvent,
    WatchEvent,
}

fn raw_github_events(json: String) -> Result<Vec<RawEvent>, Error> {
    return serde_json::from_str::<Vec<RawEvent>>(&json);
}

#[cfg(test)]
mod test {
    use Config;
    use chrono::{TimeZone, Utc};

    #[test]
    fn parse_config_with_no_params() {
        assert_eq!(
            Config::new(&vec!["".to_string()]),
            Err("Not enough arguments, expecting at least 1 argument")
        );
    }

    #[test]
    fn parse_config_with_repo_param() {
        assert_eq!(
            Config::new(&vec!["".to_string(), "fakeRepo".to_string()]),
            Ok(Config {
                repo: "fakeRepo".to_string(),
                token: None,
            })
        );
    }

    #[test]
    fn parse_config_with_repo_and_token_params() {
        assert_eq!(
            Config::new(&vec![
                "".to_string(),
                "fakeRepo".to_string(),
                "fakeToken".to_string(),
            ]),
            Ok(Config {
                repo: "fakeRepo".to_string(),
                token: Some("fakeToken".to_string()),
            })
        );
    }

    use {raw_github_events, Action, Actor, Payload, RawEvent, Type};

    #[test]
    fn parse_github_events() {
        assert_eq!(
            raw_github_events(include_str!("../test/github_events.json").to_string()).unwrap()[0],
            RawEvent {
                actor: Actor {
                    login: "alice".to_string(),
                },
                payload: Payload {
                    action: Some(Action::opened),
                },
                event_type: Type::PullRequestEvent,
                created_at: Utc.ymd(2016, 12, 1).and_hms(16, 26, 43),
            }
        );
    }

    #[test]
    fn parse_real_github_events_from_nicokosi_pullpito() {
        let events =
            raw_github_events(include_str!("../test/pullpito_github_events.json").to_string());
        assert!(events.is_ok());
    }

    #[test]
    fn parse_real_github_events_from_python_peps() {
        let events =
            raw_github_events(include_str!("../test/python_peps_github_events.json").to_string());
        assert!(events.is_ok());
    }
}
