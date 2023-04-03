use std::io::{Error, ErrorKind};
use std::str;

use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use log::debug;
use log::trace;
use regex::Regex;
use reqwest::header;
use serde::{Deserialize, Deserializer};

pub(crate) fn github_events(repo: &str, token: &Option<String>) -> Result<Vec<RawEvent>, Error> {
    let mut raw_events: Vec<RawEvent> = Vec::new();
    for page in 1..10 {
        let token = token.clone();
        let url = format!("https://api.github.com/repos/{repo}/events?page={page}");
        let mut headers = header::HeaderMap::new();
        if token.is_some() {
            let mut value = "token ".to_string();
            value.push_str(&token.unwrap_or_default());
            headers.insert(header::AUTHORIZATION, value.parse().unwrap());
        }
        let mut resp = reqwest::Client::new()
            .get(url.as_str())
            .headers(headers)
            .send()
            .expect("Cannot connect to GitHub API");
        let body = match resp.text() {
            Ok(body) => {
                if body.len() <= "[]".len() {
                    debug!("No more content for {:?} (page number: {})", repo, page);
                    break;
                }
                debug!("Content found for {:?} (page number: {})", repo, page);
                trace!(
                    "Content found for {:?} (page number: {}): {:?}",
                    repo,
                    page,
                    body
                );
                body
            }
            Err(error) => {
                if error.status() == Some(reqwest::StatusCode::UNPROCESSABLE_ENTITY) {
                    debug!("No more content for {:?} (page number: {})", repo, page);
                    break;
                }
                debug!("Oops, something went wrong with GitHub API {:?}", error);
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("Cannot get GitHub API content: {error}"),
                ));
            }
        };

        let mut raw_events_per_page =
            raw_github_events(&body).expect("Cannot deserialize GitHub API content");
        raw_events.append(&mut raw_events_per_page);

        // Stop iterating on event pages if current page is the last one
        match resp.headers().get("Link").as_ref() {
            Some(link_header) => {
                let link_header = link_header.as_bytes();
                let last_page = last_page_from_link_header(str::from_utf8(link_header).unwrap());
                debug!("Last page: {:?} (current page: {})", last_page, page);
                match last_page {
                    Some(last_page) => {
                        if page == last_page {
                            break;
                        }
                    }
                    None => break,
                }
            }
            None => break,
        }
    }
    Ok(raw_events)
}

fn raw_github_events(json: &str) -> Result<Vec<RawEvent>, serde_json::Error> {
    debug!("{}", json);
    serde_json::from_str::<Vec<RawEvent>>(json)
}

fn last_page_from_link_header(link_header: &str) -> Option<u32> {
    lazy_static! {
        static ref RE: Regex = Regex::new(".*page=(\\d+)>; rel=\"last\".*").unwrap();
    }
    RE.captures(link_header).map(|c| c[1].parse().unwrap())
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct RawEvent {
    pub actor: Actor,
    pub payload: Payload,
    #[serde(
        rename = "type",
        deserialize_with = "deserialize_field_type",
        default = "Type::default"
    )]
    pub event_type: Type,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct Actor {
    pub login: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct Payload {
    #[serde(
        deserialize_with = "deserialize_field_action",
        default = "Action::default"
    )]
    pub action: Action,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub enum Action {
    #[allow(non_camel_case_types)]
    created,
    #[allow(non_camel_case_types)]
    closed,
    #[allow(non_camel_case_types)]
    opened,
    #[serde(skip_deserializing)]
    Unknown,
}
impl Action {
    const fn default() -> Self {
        Self::Unknown
    }
}

fn deserialize_field_action<'de, D>(deserializer: D) -> Result<Action, D::Error>
where
    D: Deserializer<'de>,
{
    Action::deserialize(deserializer).or(Ok(Action::Unknown))
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub enum Type {
    IssueCommentEvent,
    PullRequestEvent,
    PullRequestReviewCommentEvent,
    #[serde(skip_deserializing)]
    Unknown,
}
impl Type {
    const fn default() -> Self {
        Self::Unknown
    }
}

fn deserialize_field_type<'de, D>(deserializer: D) -> Result<Type, D::Error>
where
    D: Deserializer<'de>,
{
    Type::deserialize(deserializer).or(Ok(Type::Unknown))
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    #[test]
    fn parse_github_events() {
        assert_eq!(
            raw_github_events(include_str!("../../test/github_events.json")).unwrap()[0],
            RawEvent {
                actor: Actor {
                    login: "alice".to_string(),
                },
                payload: Payload {
                    action: Action::opened,
                },
                event_type: Type::PullRequestEvent,
                created_at: Utc.with_ymd_and_hms(2016, 12, 1, 16, 26, 43).unwrap(),
            }
        );
    }

    #[test]
    fn parse_github_event_with_unknown_enums() {
        assert_eq!(
            raw_github_events(include_str!(
                "../../test/github_event_with_unknown_enums.json"
            ))
            .unwrap()[0],
            RawEvent {
                actor: Actor {
                    login: "alice".to_string(),
                },
                payload: Payload {
                    action: Action::Unknown,
                },
                event_type: Type::Unknown,
                created_at: Utc.with_ymd_and_hms(2016, 12, 1, 16, 26, 43).unwrap(),
            }
        );
    }

    #[test]
    fn parse_real_github_events_from_nicokosi_pullpito() {
        let events = raw_github_events(include_str!("../../test/pullpito_github_events.json"));
        assert!(events.is_ok());
    }

    #[test]
    fn parse_real_github_events_from_python_peps() {
        let events = raw_github_events(include_str!("../../test/python_peps_github_events.json"));
        assert!(events.is_ok());
    }

    #[test]
    fn parse_github_link_header_for_page_1_over_10() {
        let last_page = last_page_from_link_header(
            "<https://api.github.com/repositories/257951013/events?page=2>; rel=\"next\", <https://api.github.com/repositories/257951013/events?page=10>; rel=\"last\"");
        assert_eq!(last_page, Some(10));
    }

    #[test]
    fn parse_github_link_header_for_page_2_over_10() {
        let last_page = last_page_from_link_header(
            "<https://api.github.com/repositories/257951013/events?page=1>; rel=\"prev\", <https://api.github.com/repositories/257951013/events?page=3>; rel=\"next\", <https://api.github.com/repositories/257951013/events?page=10>; rel=\"last\", <https://api.github.com/repositories/257951013/events?page=1>; rel=\"first\"");
        assert_eq!(last_page, Some(10));
    }

    #[test]
    fn parse_github_link_header_for_page_10_over_10() {
        let last_page = last_page_from_link_header(
            "<https://api.github.com/repositories/257951013/events?page=9>; rel=\"prev\", <https://api.github.com/repositories/257951013/events?page=1>; rel=\"first\"");
        assert_eq!(last_page, None);
    }

    #[test]
    fn parse_github_link_header_can_fail_because_of_unknown_header_value() {
        let last_page = last_page_from_link_header("moo");
        assert_eq!(last_page, None);
    }
}
