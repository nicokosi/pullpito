use std::error::Error;

use reqwest::StatusCode;
use reqwest::header;
use serde::Deserialize;

pub(crate) fn github_events(
    repo: &str,
    token: Option<&str>,
) -> Result<Vec<RawEvent>, Box<dyn Error + Send + Sync>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("nicokosi/pullpito")
        .build()?;
    let mut raw_events = Vec::new();

    for page in 1..10 {
        let url = format!("https://api.github.com/repos/{repo}/events?page={page}");
        let mut request = client.get(&url);
        if let Some(token) = token {
            request = request.header(header::AUTHORIZATION, format!("token {token}"));
        }

        let response = request.send()?;
        if response.status() == StatusCode::UNPROCESSABLE_ENTITY {
            break;
        }
        let response = response.error_for_status()?;
        let headers = response.headers().clone();
        let body = response.text()?;
        if body.len() <= "[]".len() {
            break;
        }

        raw_events.extend(serde_json::from_str::<Vec<RawEvent>>(&body)?);
        let Some(link_header) = headers.get(header::LINK) else {
            break;
        };
        let Some(link_header) = link_header.to_str().ok() else {
            break;
        };
        let Some(last_page) = last_page_from_link_header(link_header) else {
            break;
        };
        if page == last_page {
            break;
        }
    }
    Ok(raw_events)
}

fn last_page_from_link_header(link_header: &str) -> Option<u32> {
    link_header.split(',').find_map(|link| {
        let (url, params) = link.split_once('>')?;
        if !params.contains("rel=\"last\"") {
            return None;
        }

        url.split_once('?')?.1.split('&').find_map(|parameter| {
            let (name, value) = parameter.split_once('=')?;
            (name == "page").then(|| value.parse().ok()).flatten()
        })
    })
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct RawEvent {
    pub actor: Actor,
    pub payload: Payload,
    #[serde(rename = "type", default)]
    pub event_type: Type,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct Actor {
    pub login: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct Payload {
    #[serde(default)]
    pub action: Action,
}

#[derive(Debug, Default, Deserialize, Eq, PartialEq)]
pub enum Action {
    #[allow(non_camel_case_types)]
    created,
    #[allow(non_camel_case_types)]
    closed,
    #[allow(non_camel_case_types)]
    opened,
    #[default]
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Default, Deserialize, Eq, PartialEq)]
pub enum Type {
    IssueCommentEvent,
    PullRequestEvent,
    PullRequestReviewCommentEvent,
    #[default]
    #[serde(other)]
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_github_events() {
        let event: RawEvent =
            serde_json::from_str::<Vec<RawEvent>>(include_str!("../../test/github_events.json"))
                .unwrap()
                .remove(0);
        assert_eq!(
            event,
            RawEvent {
                actor: Actor {
                    login: "alice".to_owned(),
                },
                payload: Payload {
                    action: Action::opened,
                },
                event_type: Type::PullRequestEvent,
                created_at: "2016-12-01T16:26:43Z".to_owned(),
            }
        );
    }

    #[test]
    fn parse_github_event_with_unknown_enums() {
        let event: RawEvent = serde_json::from_str::<Vec<RawEvent>>(include_str!(
            "../../test/github_event_with_unknown_enums.json"
        ))
        .unwrap()
        .remove(0);
        assert_eq!(event.payload.action, Action::Unknown);
        assert_eq!(event.event_type, Type::Unknown);
    }

    #[test]
    fn parse_real_github_event_fixtures() {
        for fixture in [
            include_str!("../../test/pullpito_github_events.json"),
            include_str!("../../test/python_peps_github_events.json"),
        ] {
            let events: Vec<RawEvent> = serde_json::from_str(fixture).unwrap();
            assert!(!events.is_empty());
        }
    }

    #[test]
    fn parse_github_link_header_for_page_1_over_10() {
        let link_header = "<https://api.github.com/repositories/257951013/events?page=2>; rel=\"next\", <https://api.github.com/repositories/257951013/events?page=10>; rel=\"last\"";
        assert_eq!(last_page_from_link_header(link_header), Some(10));
    }

    #[test]
    fn parse_github_link_header_for_page_2_over_10() {
        let link_header = "<https://api.github.com/repositories/257951013/events?page=1>; rel=\"prev\", <https://api.github.com/repositories/257951013/events?page=3>; rel=\"next\", <https://api.github.com/repositories/257951013/events?page=10>; rel=\"last\", <https://api.github.com/repositories/257951013/events?page=1>; rel=\"first\"";
        assert_eq!(last_page_from_link_header(link_header), Some(10));
    }

    #[test]
    fn parse_github_link_header_with_additional_query_parameters() {
        let link_header = "<https://api.github.com/repositories/257951013/events?page=2>; rel=\"next\", <https://api.github.com/repositories/257951013/events?per_page=100&page=42>; rel=\"last\"";
        assert_eq!(last_page_from_link_header(link_header), Some(42));
    }

    #[test]
    fn parse_github_link_header_without_last_page() {
        assert_eq!(
            last_page_from_link_header(
                "<https://api.github.com/repositories/257951013/events?page=9>; rel=\"prev\", <https://api.github.com/repositories/257951013/events?page=1>; rel=\"first\""
            ),
            None
        );
        assert_eq!(last_page_from_link_header("moo"), None);
    }
}
