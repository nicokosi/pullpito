use std::collections::HashMap;
use std::ffi::OsString;
use std::fmt::Write as _;
use std::str;
use std::sync::mpsc;
use std::thread;

use log::debug;
use log::info;
use serde::Deserialize;
use serde::Serialize;
use structopt::StructOpt;

use crate::github_events::{github_events as _github_events, Action, RawEvent, Type};

pub mod github_events;

#[derive(Debug, Deserialize, PartialEq, Eq, Serialize)]
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
        required = true,
        takes_value = true,
        multiple = true,
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

fn config_from_args(args: Vec<OsString>) -> Config {
    let options = Options::from_iter(args);
    Config {
        repos: options.repositories,
        token: options.token,
    }
}

/// Calls GitHub REST API in order to log pull requests' statistics in the standard output.
///
/// # Panics
///
/// Panics if the GitHub API request fails or if response cannot be deserialized.
pub fn log_github_events(os: Vec<OsString>) {
    env_logger::init();
    let config = config_from_args(os);
    info!(
        "Computing stats for GitHub repos '{:?}' (with token: {})",
        config.repos,
        config.token.is_some()
    );

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
            print_events_per_author(&repo_events.repo, &repo_events.events_per_author)
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

fn print_events_per_author(
    repo: &str,
    events_per_author: &HashMap<String, Vec<RawEvent>>,
) -> String {
    let mut out: String = format!("pull requests for {repo:?} ->\n");
    out.push_str("  opened per author:\n");
    print_pull_request_events_per_author(events_per_author, Action::opened, &mut out);
    out.push_str("  commented per author:\n");
    print_pull_request_events_per_author(events_per_author, Action::created, &mut out);
    out.push_str("  closed per author:\n");
    print_pull_request_events_per_author(events_per_author, Action::closed, &mut out);
    out
}

fn print_pull_request_events_per_author(
    events_per_author: &HashMap<String, Vec<RawEvent>>,
    payload_action: Action,
    out: &mut String,
) {
    for (author, events) in events_per_author.iter() {
        let matching_pull_requests = events
            .iter()
            .filter(|e| {
                e.event_type == Type::PullRequestEvent && e.payload.action == payload_action
            })
            .count();
        if matching_pull_requests > 0 {
            let _ = writeln!(out, "    {author}: {matching_pull_requests}");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::{TimeZone, Utc};

    use crate::config_from_args;
    use crate::events_per_author;
    use crate::print_events_per_author;
    use crate::Config;
    use crate::OsString;

    use super::github_events::*;

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
                created_at: Utc.with_ymd_and_hms(2016, 12, 1, 16, 26, 43).unwrap(),
            }],
        );

        let printable = print_events_per_author("my-org/my-repo", &events);

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
            created_at: Utc.with_ymd_and_hms(2016, 12, 1, 16, 26, 43).unwrap(),
        }]);
        assert_eq!(events_per_author.get("alice").iter().len(), 1);
    }
}
