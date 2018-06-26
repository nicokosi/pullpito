extern crate reqwest;
extern crate serde_json;

use chrono::{DateTime, Utc};
use std::io::{Error, ErrorKind};

pub fn github_events(repo: &str, token: Option<String>) -> Result<Vec<RawEvent>, Error> {
    let url = format!(
        "https://api.github.com/repos/{}/events?access_token={}&page=1",
        repo,
        &token.unwrap_or_else(|| "".to_string())
    );
    let resp = reqwest::get(url.as_str());
    let mut resp = match resp {
        Ok(resp) => resp,
        Err(_) => return Err(Error::new(ErrorKind::Other, "Cannot connect to GitHub API")),
    };
    let body = resp.text();
    let body = match body {
        Ok(body) => body,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::Other,
                "Cannot get GitHub API content",
            ))
        }
    };
    let raw_events = raw_github_events(&body);
    let raw_events = match raw_events {
        Ok(events) => events,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::Other,
                "Cannot deserialize GitHub API content",
            ))
        }
    };
    Ok(raw_events)
}

fn raw_github_events(json: &str) -> Result<Vec<RawEvent>, serde_json::Error> {
    serde_json::from_str::<Vec<RawEvent>>(&json)
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
            raw_github_events(include_str!("../../test/github_events.json")).unwrap()
                [0],
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
            raw_github_events(include_str!("../../test/pullpito_github_events.json"));
        assert!(events.is_ok());
    }

    #[test]
    fn parse_real_github_events_from_python_peps() {
        let events = raw_github_events(
            include_str!("../../test/python_peps_github_events.json"),
        );
        assert!(events.is_ok());
    }

}
