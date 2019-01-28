extern crate chrono;
extern crate env_logger;
extern crate futures;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate regex;
#[macro_use]
extern crate serde_derive;
extern crate structopt;

use crate::github_events::{github_events as _github_events, Action, RawEvent, Type};
use std::collections::HashMap;
use std::ffi::OsString;
use std::str;
use std::sync::mpsc;
use std::thread;
use structopt::StructOpt;

pub mod github_events;
extern crate serde;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Config {
    pub repos: Vec<String>,
    pub token: Option<String>,
}

#[derive(StructOpt, Debug)]
#[structopt(
    about = "Display simple counters for GitHub Pull Requests",
    author = "Nicolas Kosinski <nicokosi@yahoo.com>",
    name = "Pullpito 🐙",
    version = "0.1.0"
)]
struct Options {
    #[structopt(
        long = "repository",
        help = "the name of a GitHub repository, i.e. 'python/peps'",
        raw(required = "true"),
        raw(takes_value = "true"),
        short = "r"
    )]
    repositories: Vec<String>,

    #[structopt(
        help = "an optional GitHub personal access token (required for private GitHub repositories)",
        long = "token",
        short = "t"
    )]
    token: Option<String>,
}

pub fn config_from_args(args: Vec<OsString>) -> Config {
    let options = Options::from_iter(args);
    Config {
        repos: options.repositories,
        token: options.token,
    }
}

pub fn github_events(config: Config) {
    let (sender, receiver) = mpsc::channel();
    let number_of_repos = config.repos.len();

    for repo in config.repos {
        debug!("Query stats for GitHub repo {:?}", repo);
        let sender = mpsc::Sender::clone(&sender);
        let token = config.token.clone();
        thread::spawn(move || {
            sender
                .send(RepoEvents {
                    repo: repo.clone(),
                    events_per_author: events_per_author(_github_events(&repo, &token).unwrap()),
                })
                .unwrap();
        });
    }

    for _ in 0..number_of_repos {
        let repo_events = receiver.recv().unwrap();
        debug!("Print stats for GitHub repo {:?}", repo_events.repo);
        println!(
            "{}",
            printable(&repo_events.repo, &repo_events.events_per_author)
        );
    }
}

struct RepoEvents {
    repo: String,
    events_per_author: HashMap<String, Vec<RawEvent>>,
}

fn events_per_author(events: Vec<RawEvent>) -> HashMap<String, Vec<RawEvent>> {
    events
        .into_iter()
        .filter(|e| {
            e.event_type == Type::PullRequestEvent
                || e.event_type == Type::PullRequestReviewCommentEvent
                || e.event_type == Type::IssueCommentEvent
        })
        .fold(HashMap::new(), |mut acc, event: RawEvent| {
            (*acc
                .entry(event.actor.login.clone())
                .or_insert_with(Vec::new))
            .push(event);
            acc
        })
}

fn printable(repo: &str, events_per_author: &HashMap<String, Vec<RawEvent>>) -> String {
    let mut out: String = format!("pull requests for {:?} ->\n", repo);
    out.push_str("  opened per author:\n");
    for (author, events) in events_per_author.iter() {
        let opened_pull_requests = events
            .iter()
            .filter(|e| {
                e.event_type == Type::PullRequestEvent && e.payload.action == Action::opened
            })
            .count();
        if opened_pull_requests > 0 {
            out.push_str(&format!("    {}: {}\n", author, opened_pull_requests));
        }
    }
    out.push_str("  commented per author:\n");
    for (author, events) in events_per_author.iter() {
        let commented_pull_requests = events
            .iter()
            .filter(|e| {
                e.event_type == Type::IssueCommentEvent && e.payload.action == Action::created
            })
            .count();
        if commented_pull_requests > 0 {
            out.push_str(&format!("    {}: {}\n", author, commented_pull_requests));
        }
    }
    out.push_str("  closed per author:\n");
    for (author, events) in events_per_author.iter() {
        let closed_pull_requests = events
            .iter()
            .filter(|e| {
                e.event_type == Type::PullRequestEvent && e.payload.action == Action::closed
            })
            .count();
        if closed_pull_requests > 0 {
            out.push_str(&format!("    {}: {}\n", author, closed_pull_requests));
        }
    }
    out.to_string()
}

#[cfg(test)]
mod test {
    use crate::config_from_args;
    use crate::events_per_author;
    use crate::github_events::{Action, Actor, Payload, RawEvent, Type};
    use crate::printable;
    use crate::Config;
    use crate::OsString;
    use chrono::{TimeZone, Utc};
    use std::collections::HashMap;

    #[test]
    fn parse_args_with_a_long_repo_param() {
        assert_eq!(
            config_from_args(vec![
                OsString::from("pullpito"),
                OsString::from("--repository"),
                OsString::from("fakeRepo"),
            ]),
            Config {
                repos: vec!["fakeRepo".to_string()],
                token: None,
            },
        );
    }

    #[test]
    fn parse_args_with_a_long_repo_param_and_a_long_token_param() {
        assert_eq!(
            config_from_args(vec![
                OsString::from("pullpito"),
                OsString::from("--repository"),
                OsString::from("fakeRepo"),
                OsString::from("--token"),
                OsString::from("fakeToken"),
            ]),
            Config {
                repos: vec!["fakeRepo".to_string()],
                token: Some("fakeToken".to_string()),
            }
        );
    }

    #[test]
    fn parse_args_with_two_long_repo_params_and_a_long_token_param() {
        assert_eq!(
            config_from_args(vec![
                OsString::from("pullpito"),
                OsString::from("--repository"),
                OsString::from("fakeRepo1"),
                OsString::from("--repository"),
                OsString::from("fakeRepo2"),
                OsString::from("--token"),
                OsString::from("fakeToken"),
            ]),
            Config {
                repos: vec!["fakeRepo1".to_string(), "fakeRepo2".to_string()],
                token: Some("fakeToken".to_string()),
            }
        );
    }

    #[test]
    fn parse_args_with_two_short_repo_params_and_a_short_token_param() {
        assert_eq!(
            config_from_args(vec![
                OsString::from("pullpito"),
                OsString::from("-r"),
                OsString::from("fakeRepo1"),
                OsString::from("-r"),
                OsString::from("fakeRepo2"),
                OsString::from("-t"),
                OsString::from("fakeToken"),
            ]),
            Config {
                repos: vec!["fakeRepo1".to_string(), "fakeRepo2".to_string()],
                token: Some("fakeToken".to_string()),
            }
        );
    }

    #[test]
    fn printable_with_opened_pull_request() {
        let mut events: HashMap<String, Vec<RawEvent>> = HashMap::new();
        events.insert(
            "alice".to_string(),
            vec![RawEvent {
                actor: Actor {
                    login: "alice".to_string(),
                },
                payload: Payload {
                    action: Action::opened,
                },
                event_type: Type::PullRequestEvent,
                created_at: Utc.ymd(2016, 12, 1).and_hms(16, 26, 43),
            }],
        );

        let printable = printable("my-org/my-repo", &events);

        assert!(printable.contains("pull requests for \"my-org/my-repo\" ->"));
        assert!(printable.contains("opened per author:\n    alice: 1\n"));
        assert!(printable.contains("opened per author:\n    alice: 1\n"));
        assert!(printable.contains("commented per author:\n  closed per author:\n"));
    }

    #[test]
    fn compute_events_per_author() {
        let events_per_author = events_per_author(vec![RawEvent {
            actor: Actor {
                login: "alice".to_string(),
            },
            payload: Payload {
                action: Action::opened,
            },
            event_type: Type::PullRequestEvent,
            created_at: Utc.ymd(2016, 12, 1).and_hms(16, 26, 43),
        }]);
        assert_eq!(events_per_author.get("alice").iter().len(), 1);
    }

}
