extern crate chrono;
extern crate env_logger;
extern crate futures;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate log;

pub mod github_events;
#[macro_use]
extern crate lazy_static;
extern crate regex;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Config {
    pub repos: Vec<String>,
    pub token: Option<String>,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 2 {
            return Err(
                "Missing arguments.\n\n \
                 Usage: pullpito $repositories $token\n\n \
                 \t$repositories: a comma-separated list of GitHub repositories. Examples:\n \
                 \t\tpython/peps\n \
                 \t\tpython/peps,rust-lang/rust\n\n \
                 \t$token: an optional GitHub personal access token",
            );
        }
        let repos = args[1].clone();
        let token = if args.len() == 3 {
            Some(args[2].clone())
        } else {
            None
        };
        let repos = repos.split(',').map(|s| s.to_string()).collect();
        Ok(Config { repos, token })
    }
}

use std::str;
use std::collections::HashMap;
use std::thread;
use std::sync::mpsc;
use github_events::{github_events as _github_events, Action, RawEvent, Type};

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
                    events_per_author: events_per_author(
                        _github_events(&repo, &token).unwrap(),
                    ),
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
            (*acc.entry(event.actor.login.clone())
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
    out.to_string()
}

#[cfg(test)]
mod test {
    use Config;

    #[test]
    fn parse_config_with_no_params() {
        let config = Config::new(&vec!["".to_string()]);
        assert!(config.is_err());
        assert!(config.unwrap_err().contains("Missing arguments."));
    }

    #[test]
    fn parse_config_with_repo_param() {
        assert_eq!(
            Config::new(&vec!["".to_string(), "fakeRepo".to_string()]),
            Ok(Config {
                repos: vec!["fakeRepo".to_string()],
                token: None,
            })
        );
    }

    #[test]
    fn parse_config_with_one_repo_and_token_params() {
        assert_eq!(
            Config::new(&vec![
                "".to_string(),
                "fakeRepo".to_string(),
                "fakeToken".to_string(),
            ]),
            Ok(Config {
                repos: vec!["fakeRepo".to_string()],
                token: Some("fakeToken".to_string()),
            })
        );
    }

    #[test]
    fn parse_config_with_two_repos_and_token_params() {
        assert_eq!(
            Config::new(&vec![
                "".to_string(),
                "fakeRepo1,fakeRepo2".to_string(),
                "fakeToken".to_string(),
            ]),
            Ok(Config {
                repos: vec!["fakeRepo1".to_string(), "fakeRepo2".to_string()],
                token: Some("fakeToken".to_string()),
            })
        );
    }

    use std::collections::HashMap;
    use printable;
    use github_events::{Action, Actor, Payload, RawEvent, Type};
    use chrono::{TimeZone, Utc};

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

        let printable = printable("my-org/my-repo", &events);

        assert!(printable.contains("pull requests for \"my-org/my-repo\" ->"));
        assert!(printable.contains("opened per author:\n    alice: 1\n"));
        assert!(printable.contains("opened per author:\n    alice: 1\n"));
        assert!(printable.contains("commented per author:\n  closed per author:\n"));
    }

    use events_per_author;

    #[test]
    fn compute_events_per_author() {
        let events_per_author = events_per_author(vec![
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
        ]);
        assert_eq!(events_per_author.get("alice").into_iter().len(), 1);
    }

}
