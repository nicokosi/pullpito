extern crate chrono;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate regex;
extern crate serde_json;
extern crate tokio_core;

#[macro_use]
extern crate log;
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
use regex::Regex;

pub fn github_events(config: Config) {
    let mut core = Core::new().unwrap();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &core.handle()).unwrap())
        .build(&core.handle());

    let url = "https://api.github.com/repos/".to_string() + &config.repo
        + "/events?&page=1&access_token=" + &config.token.unwrap_or("".to_string());
    let mut req = Request::new(Method::Get, url.parse().unwrap());
    req.headers_mut()
        .set_raw("Accept", "application/vnd.github.v3+json");
    req.headers_mut().set_raw("Host", "api.github.com");
    req.headers_mut().set_raw("User-Agent", "pullpito/0.1.0");

    let safe_url = Regex::new(r"(?P<safe_url>\w+)&access_token=(?P<token>\w+)").unwrap();
    let unsafe_url = req.uri().to_string();
    debug!(
        "GitHub request: {:?}",
        safe_url.replace(&unsafe_url, "${safe_url}")
    );

    let repo = config.repo;

    let work = client.request(req).and_then(|res| {
        res.body().concat2().and_then(move |body| {
            let raw_events_as_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
            let raw_events = raw_github_events(raw_events_as_json.to_string());
            let events_per_author: HashMap<String, Vec<RawEvent>> =
                events_per_author(raw_events.unwrap());
            println!("{}", printable(repo, events_per_author));
            Ok(())
        })
    });

    core.run(work).unwrap();
}

fn printable(repo: String, events_per_author: HashMap<String, Vec<RawEvent>>) -> String {
    let mut out: String = format!("pull requests for {:?} ->\n", repo);
    out.push_str("  opened per author:\n");
    for (author, events) in events_per_author.iter() {
        let opened_pull_requests = events
            .into_iter()
            .filter(|e| {
                e.event_type == Type::PullRequestEvent && e.payload.action == Some(Action::opened)
            })
            .count();
        if opened_pull_requests > 0 {
            out.push_str(&format!("    {}: {}\n", author, opened_pull_requests));
        }
    }
    out.push_str("  commented per author:\n");
    for (author, events) in events_per_author.iter() {
        let commented_pull_requests = events
            .into_iter()
            .filter(|e| {
                e.event_type == Type::IssueCommentEvent && e.payload.action == Some(Action::created)
            })
            .count();
        if commented_pull_requests > 0 {
            out.push_str(&format!("    {}: {}\n", author, commented_pull_requests));
        }
    }
    out.push_str("  closed per author:\n");
    for (author, events) in events_per_author.iter() {
        let closed_pull_requests = events
            .into_iter()
            .filter(|e| {
                e.event_type == Type::PullRequestEvent && e.payload.action == Some(Action::closed)
            })
            .count();
        if closed_pull_requests > 0 {
            out.push_str(&format!("    {}: {}\n", author, closed_pull_requests));
        }
    }
    return out.to_string();
}

use std::collections::HashMap;

fn events_per_author(events: Vec<RawEvent>) -> HashMap<String, Vec<RawEvent>> {
    return events
        .into_iter()
        .filter(|e| {
            e.event_type == Type::PullRequestEvent
                || e.event_type == Type::PullRequestReviewCommentEvent
                || e.event_type == Type::IssueCommentEvent
        })
        .fold(HashMap::new(), |mut acc, event: RawEvent| {
            (*acc.entry(event.actor.login.clone()).or_insert(Vec::new())).push(event);
            acc
        });
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct RawEvent {
    actor: Actor,
    payload: Payload,
    #[serde(rename = "type")]
    event_type: Type,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct Actor {
    login: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct Payload {
    action: Option<Action>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
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

#[derive(Debug, Deserialize, Eq, PartialEq)]
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

    use events_per_author;

    #[test]
    fn compute_events_per_author() {
        let events = events_per_author(
            raw_github_events(include_str!("../test/github_events.json").to_string()).unwrap(),
        );
        assert_eq!(events.get("alice").into_iter().len(), 1);
    }

    use std::collections::HashMap;
    use printable;

    #[test]
    fn printable_with_opened_pull_request() {
        let mut events: HashMap<String, Vec<RawEvent>> = HashMap::new();
        events.insert(
            "alice".to_string(),
            vec![
                RawEvent {
                    actor: Actor {
                        login: "alice".to_string(),
                    },
                    payload: Payload {
                        action: Some(Action::opened),
                    },
                    event_type: Type::PullRequestEvent,
                    created_at: Utc.ymd(2016, 12, 1).and_hms(16, 26, 43),
                },
            ],
        );

        let out = printable("repo".to_string(), events);

        assert!(out.contains("opened per author:\n    alice: 1"));
        assert_eq!(out.contains("commented per author:\n    alice: "), false);
        assert_eq!(out.contains("closed per author:\n    alice: "), false);
    }

    #[test]
    fn printable_with_commented_pull_request() {
        let mut events: HashMap<String, Vec<RawEvent>> = HashMap::new();
        events.insert(
            "alice".to_string(),
            vec![
                RawEvent {
                    actor: Actor {
                        login: "alice".to_string(),
                    },
                    payload: Payload {
                        action: Some(Action::created),
                    },
                    event_type: Type::IssueCommentEvent,
                    created_at: Utc.ymd(2016, 12, 1).and_hms(16, 26, 43),
                },
            ],
        );

        let out = printable("repo".to_string(), events);

        assert_eq!(out.contains("opened per author:\n    alice: "), false);
        assert!(out.contains("commented per author:\n    alice: 1"));
        assert_eq!(out.contains("closed per author:\n    alice: "), false);
    }

    #[test]
    fn printable_with_closed_pull_request() {
        let mut events: HashMap<String, Vec<RawEvent>> = HashMap::new();
        events.insert(
            "alice".to_string(),
            vec![
                RawEvent {
                    actor: Actor {
                        login: "alice".to_string(),
                    },
                    payload: Payload {
                        action: Some(Action::closed),
                    },
                    event_type: Type::PullRequestEvent,
                    created_at: Utc.ymd(2016, 12, 1).and_hms(16, 26, 43),
                },
            ],
        );

        let out = printable("repo".to_string(), events);

        assert_eq!(out.contains("opened per author:\n    alice: "), false);
        assert_eq!(out.contains("commented per author:\n    alice: "), false);
        assert!(out.contains("closed per author:\n    alice: 1"));
    }

}
