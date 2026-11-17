use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsString;
use std::fmt::Write as _;
use std::io;
use std::sync::mpsc;

use crate::github_events::{Action, RawEvent, Type, github_events};

pub mod github_events;

#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub repos: Vec<String>,
    pub token: Option<String>,
}

fn config_from_args(args: impl IntoIterator<Item = OsString>) -> Result<Config, String> {
    let mut args = args.into_iter();
    args.next();

    let mut repos = Vec::new();
    let mut token = None;
    while let Some(argument) = args.next() {
        let argument = argument
            .into_string()
            .map_err(|_| "arguments must be valid UTF-8".to_owned())?;
        let (option, inline_value) = argument
            .split_once('=')
            .map_or((argument.as_str(), None), |(option, value)| {
                (option, Some(value.to_owned()))
            });

        match option {
            "--repository" | "-r" => {
                repos.push(option_value(option, inline_value, &mut args)?);
            }
            "--token" | "-t" => {
                token = Some(option_value(option, inline_value, &mut args)?);
            }
            _ => return Err(format!("unknown argument: {argument}")),
        }
    }

    if repos.is_empty() {
        return Err("at least one --repository (-r) is required".to_owned());
    }

    Ok(Config { repos, token })
}

fn option_value(
    option: &str,
    inline_value: Option<String>,
    args: &mut impl Iterator<Item = OsString>,
) -> Result<String, String> {
    let value = match inline_value {
        Some(value) => value,
        None => args
            .next()
            .ok_or_else(|| format!("missing value for {option}"))?
            .into_string()
            .map_err(|_| format!("value for {option} must be valid UTF-8"))?,
    };
    if value.is_empty() {
        return Err(format!("value for {option} cannot be empty"));
    }
    Ok(value)
}

/// Fetches GitHub events and prints pull request counters for each repository.
pub fn run(args: impl IntoIterator<Item = OsString>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let args: Vec<_> = args.into_iter().collect();
    if args
        .iter()
        .skip(1)
        .any(|arg| arg == "--help" || arg == "-h")
    {
        print_help();
        return Ok(());
    }
    if args
        .iter()
        .skip(1)
        .any(|arg| arg == "--version" || arg == "-V")
    {
        println!("pullpito {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let config = config_from_args(args).map_err(io::Error::other)?;
    let (sender, receiver) = mpsc::channel();
    let number_of_repos = config.repos.len();

    std::thread::scope(|scope| -> Result<(), Box<dyn Error + Send + Sync>> {
        for repo in config.repos {
            let sender = sender.clone();
            let token = config.token.as_deref();
            scope.spawn(move || {
                let result = github_events(&repo, token).map(|events| RepoEvents {
                    repo,
                    events_per_author: events_per_author(events),
                });
                let _ = sender.send(result);
            });
        }
        drop(sender);

        for _ in 0..number_of_repos {
            let repo_events = receiver.recv().map_err(io::Error::other)??;
            println!(
                "{}",
                print_events_per_author(&repo_events.repo, &repo_events.events_per_author)
            );
        }
        Ok(())
    })
}

fn print_help() {
    println!(
        "Pullpito 🐙 {}\nDisplay simple counters for GitHub Pull Requests\n\n\
         Usage: pullpito --repository <ORG/REPO>... [--token <TOKEN>]\n\n\
         Options:\n  \
         -r, --repository <ORG/REPO>  GitHub repository (repeatable)\n  \
         -t, --token <TOKEN>          GitHub token for private repositories\n  \
         -h, --help                   Print help\n  \
         -V, --version                Print version",
        env!("CARGO_PKG_VERSION")
    );
}

struct RepoEvents {
    repo: String,
    events_per_author: HashMap<String, Vec<RawEvent>>,
}

fn events_per_author(events: Vec<RawEvent>) -> HashMap<String, Vec<RawEvent>> {
    let mut grouped = HashMap::new();
    for event in events.into_iter().filter(|event| {
        matches!(
            event.event_type,
            Type::PullRequestEvent | Type::PullRequestReviewCommentEvent | Type::IssueCommentEvent
        )
    }) {
        grouped
            .entry(event.actor.login.clone())
            .or_insert_with(Vec::new)
            .push(event);
    }
    grouped
}

fn print_events_per_author(
    repo: &str,
    events_per_author: &HashMap<String, Vec<RawEvent>>,
) -> String {
    let mut output = format!("pull requests for {repo:?} ->\n");
    output.push_str("  opened per author:\n");
    print_pull_request_events_per_author(events_per_author, &Action::opened, &mut output);
    output.push_str("  commented per author:\n");
    print_pull_request_events_per_author(events_per_author, &Action::created, &mut output);
    output.push_str("  closed per author:\n");
    print_pull_request_events_per_author(events_per_author, &Action::closed, &mut output);
    output
}

fn print_pull_request_events_per_author(
    events_per_author: &HashMap<String, Vec<RawEvent>>,
    payload_action: &Action,
    output: &mut String,
) {
    for (author, events) in events_per_author {
        let count = events
            .iter()
            .filter(|event| {
                event.event_type == Type::PullRequestEvent
                    && &event.payload.action == payload_action
            })
            .count();
        if count > 0 {
            let _ = writeln!(output, "    {author}: {count}");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::github_events::{Action, Actor, Payload, RawEvent, Type};
    use crate::{Config, config_from_args, events_per_author, print_events_per_author};

    #[test]
    fn parse_repeated_repositories_and_token() {
        let config = config_from_args([
            "pullpito".into(),
            "--repository".into(),
            "org/one".into(),
            "-r".into(),
            "org/two".into(),
            "--token=secret".into(),
        ])
        .unwrap();
        assert_eq!(
            config,
            Config {
                repos: vec!["org/one".to_owned(), "org/two".to_owned()],
                token: Some("secret".to_owned()),
            }
        );
    }

    #[test]
    fn rejects_missing_repository() {
        assert!(config_from_args(["pullpito".into()]).is_err());
    }

    #[test]
    fn printable_with_opened_pull_request() {
        let events = HashMap::from([(
            "alice".to_owned(),
            vec![RawEvent {
                actor: Actor {
                    login: "alice".to_owned(),
                },
                payload: Payload {
                    action: Action::opened,
                },
                event_type: Type::PullRequestEvent,
                created_at: "2016-12-01T16:26:43Z".to_owned(),
            }],
        )]);

        let printable = print_events_per_author("my-org/my-repo", &events);
        assert!(printable.contains("pull requests for \"my-org/my-repo\" ->"));
        assert!(printable.contains("opened per author:\n    alice: 1\n"));
        assert!(printable.contains("commented per author:\n  closed per author:\n"));
    }

    #[test]
    fn groups_only_pull_request_related_events() {
        let event = RawEvent {
            actor: Actor {
                login: "alice".to_owned(),
            },
            payload: Payload {
                action: Action::opened,
            },
            event_type: Type::PullRequestEvent,
            created_at: "2016-12-01T16:26:43Z".to_owned(),
        };
        assert_eq!(events_per_author(vec![event]).len(), 1);
    }
}
