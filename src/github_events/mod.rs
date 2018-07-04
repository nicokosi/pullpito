extern crate regex;
extern crate reqwest;
extern crate serde_json;

use std::str;
use chrono::{DateTime, Utc};
use std::io::{Error, ErrorKind};
use regex::Regex;

pub fn github_events(repo: &str, token: Option<String>) -> Result<Vec<RawEvent>, Error> {
    let mut raw_events: Vec<RawEvent> = Vec::new();
    for page in 1..10 {
        let token = token.clone();
        let url = format!(
            "https://api.github.com/repos/{}/events?access_token={}&page={}",
            repo,
            &token.unwrap_or_else(|| "".to_string()),
            page,
        );
        let resp = reqwest::get(url.as_str());
        let mut resp = match resp {
            Ok(resp) => resp,
            Err(_) => return Err(Error::new(ErrorKind::Other, "Cannot connect to GitHub API")),
        };
        let body = resp.text();
        let body = match body {
            Ok(body) => {
                if body.len() <= "[]".len() {
                    debug!("No more content for {:?} (page number: {})", repo, page);
                    break;
                } else {
                    debug!("Content found for {:?} (page number: {})", repo, page);
                    trace!(
                        "Content found for {:?} (page number: {}): {:?}",
                        repo,
                        page,
                        body
                    );
                    body
                }
            }
            Err(error) => match error.status() {
                Some(reqwest::StatusCode::UnprocessableEntity) => {
                    debug!("No more content for {:?} (page number: {})", repo, page);
                    break;
                }
                _ => {
                    debug!("Oops, something went wrong with GitHub API {:?}", error);
                    return Err(Error::new(
                        ErrorKind::Other,
                        "Cannot get GitHub API content",
                    ));
                }
            },
        };

        // Stop iterating on event pages if current page is the last one
        let link_header =
            str::from_utf8(resp.headers().get_raw("Link").unwrap().one().unwrap()).unwrap();
        let last_page = last_page_from_link_header(link_header);
        trace!("Last page: {:?} (current page: {})", last_page, page);
        match last_page {
            Some(last_page) => if page == last_page {
                break;
            },
            None => break,
        }

        let raw_events_per_page = raw_github_events(&body);
        match raw_events_per_page {
            Ok(mut events) => raw_events.append(&mut events),
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    "Cannot deserialize GitHub API content",
                ))
            }
        };
    }
    Ok(raw_events)
}

fn raw_github_events(json: &str) -> Result<Vec<RawEvent>, serde_json::Error> {
    serde_json::from_str::<Vec<RawEvent>>(&json)
}

fn last_page_from_link_header(link_header: &str) -> Option<u32> {
    lazy_static! {
        static ref RE: Regex = Regex::new(".*&page=(\\d+)>; rel=\"last\".*").unwrap();
    }
    RE.captures(link_header).map(|c| c[1].parse().unwrap())
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct RawEvent {
    pub actor: Actor,
    pub payload: Payload,
    #[serde(rename = "type")]
    pub event_type: Type,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct Actor {
    pub login: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct Payload {
    pub action: Option<Action>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub enum Action {
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
pub enum Type {
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

mod tests {

    #[allow(unused_imports)] // Seems an open issue: https://github.com/rust-lang/rust/issues/43970
    use super::*;
    #[allow(unused_imports)] // Seems an open issue: https://github.com/rust-lang/rust/issues/43970
    use chrono::{TimeZone, Utc};

    #[test]
    fn parse_github_events() {
        assert_eq!(
            raw_github_events(include_str!("../../test/github_events.json")).unwrap()[0],
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
        let events = raw_github_events(include_str!("../../test/pullpito_github_events.json"));
        assert!(events.is_ok());
    }

    #[test]
    fn parse_real_github_events_from_python_peps() {
        let events = raw_github_events(include_str!("../../test/python_peps_github_events.json"));
        assert!(events.is_ok());
    }

    #[test]
    fn parse_github_link_header_for_page_1() {
        let last_page = last_page_from_link_header(
            "<https://api.github.com/repositories/128516862/events?access_token=xxx&page=2>; rel=\"next\", <https://api.github.com/repositories/128516862/events?access_token=xxx&page=5>; rel=\"last\"");
        assert_eq!(last_page, Some(5));
    }

    #[test]
    fn parse_github_link_header_for_other_pages() {
        let last_page = last_page_from_link_header(
            "<https://api.github.com/repositories/128516862/events?access_tokenxxx=&page=1>; rel=\"prev\", <https://api.github.com/repositories/128516862/events?access_token=xxx&page=3>; rel=\"next\", <https://api.github.com/repositories/128516862/events?access_token=xxx&page=5>; rel=\"last\", <https://api.github.com/repositories/128516862/events?access_token=xxx&page=1>; rel=\"first\"");
        assert_eq!(last_page, Some(5));
    }

    #[test]
    fn parse_github_link_header_can_fail_because_of_unknown_header_value() {
        let last_page = last_page_from_link_header("moo");
        assert_eq!(last_page, None);
    }

}
