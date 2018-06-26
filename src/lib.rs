extern crate chrono;
extern crate futures;

#[macro_use]
extern crate serde_derive;

pub mod github_events;

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
use std::collections::HashMap;
use github_events::{github_events as _github_events, Action, RawEvent, Type};

pub fn github_events(config: Config) {
    let raw_events = _github_events(&config.repo, config.token);
    let events_per_author: HashMap<String, Vec<RawEvent>> = events_per_author(raw_events.unwrap());
    println!("{}", printable(&config.repo, &events_per_author));
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
            (*acc.entry(event.actor.login.clone()).or_insert_with(Vec::new)).push(event);
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

        let printable = printable("foo", &events);

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
