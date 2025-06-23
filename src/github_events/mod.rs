use std::io::Error;
use std::str;

use chrono::{DateTime, Utc};
use log::debug;
use log::trace;
use reqwest::header;
use serde::{Deserialize, Deserializer};

/// Fetches repository events using GitHub's GraphQL API v4.
///
/// This function queries the GitHub GraphQL API for issue comments, pull requests, and pull request reviews
/// and converts them to RawEvent objects. It replaces the previous implementation that used GitHub's REST API v3,
/// which had limitations (no input filter, 300 events max).
///
/// # Arguments
///
/// * `repo` - A string slice containing the repository name in the format "owner/repo"
/// * `token` - An optional GitHub API token for authentication
///
/// # Returns
///
/// A Result containing a vector of RawEvent objects or an Error
pub(crate) fn github_events(repo: &str, token: &Option<String>) -> Result<Vec<RawEvent>, Error> {
    let mut raw_events: Vec<RawEvent> = Vec::new();

    // GraphQL query to fetch repository events
    let graphql_query = format!(
        r#"{{
            "query": "query {{ repository(owner: \"{owner}\", name: \"{name}\") {{ 
                issueComments(first: 100) {{ 
                    nodes {{ 
                        author {{ login }} 
                        createdAt 
                    }} 
                }} 
                pullRequests(first: 100) {{ 
                    nodes {{ 
                        author {{ login }} 
                        createdAt 
                        closedAt 
                        state 
                    }} 
                }} 
                pullRequestReviews(first: 100) {{ 
                    nodes {{ 
                        author {{ login }} 
                        createdAt 
                    }} 
                }} 
            }} }}"
        }}"#,
        owner = repo.split('/').next().unwrap_or(""),
        name = repo.split('/').nth(1).unwrap_or("")
    );

    let url = "https://api.github.com/graphql";
    let mut headers = header::HeaderMap::new();
    headers.insert(header::USER_AGENT, "nicokosi/pullpito".parse().unwrap());
    headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());

    if token.is_some() {
        let mut value = "bearer ".to_string();
        value.push_str(&token.clone().unwrap_or_default());
        headers.insert(header::AUTHORIZATION, value.parse().unwrap());
    } else {
        debug!("No token provided for GraphQL API, which may result in rate limiting");
    }

    trace!(
        "POST {}\n  headers: {:?}\n  body: {}",
        url, headers, graphql_query
    );

    let resp = reqwest::blocking::Client::new()
        .post(url)
        .headers(headers)
        .body(graphql_query)
        .send()
        .unwrap();

    let body = match resp.text() {
        Ok(body) => {
            debug!("Content found for {:?}", repo);
            trace!("Content found for {:?}: {:?}", repo, body);
            body
        }
        Err(error) => {
            debug!(
                "Oops, something went wrong with GitHub GraphQL API {:?}",
                error
            );
            return Err(Error::other(format!(
                "Cannot get GitHub GraphQL API content: {error}"
            )));
        }
    };

    // Parse GraphQL response and convert to RawEvent objects
    let graphql_events = parse_graphql_events(&body, repo)?;
    raw_events.extend(graphql_events);

    Ok(raw_events)
}

/// Parses the JSON response from GitHub's GraphQL API and converts it to RawEvent objects.
///
/// This function deserializes the GraphQL response, extracts the relevant data (issue comments,
/// pull requests, and pull request reviews), and converts it to RawEvent objects with the
/// appropriate types and actions.
///
/// # Arguments
///
/// * `json` - A string slice containing the JSON response from the GraphQL API
/// * `repo` - A string slice containing the repository name (for error reporting)
///
/// # Returns
///
/// A Result containing a vector of RawEvent objects or an Error
fn parse_graphql_events(json: &str, repo: &str) -> Result<Vec<RawEvent>, Error> {
    #[derive(Debug, Deserialize)]
    struct GraphQLResponse {
        data: Option<Data>,
        errors: Option<Vec<GraphQLError>>,
    }

    #[derive(Debug, Deserialize)]
    struct GraphQLError {
        message: String,
    }

    #[derive(Debug, Deserialize)]
    struct Data {
        repository: Option<Repository>,
    }

    #[derive(Debug, Deserialize)]
    struct Repository {
        #[serde(rename = "issueComments")]
        issue_comments: Option<NodeConnection>,
        #[serde(rename = "pullRequests")]
        pull_requests: Option<NodeConnection>,
        #[serde(rename = "pullRequestReviews")]
        pull_request_reviews: Option<NodeConnection>,
    }

    #[derive(Debug, Deserialize)]
    struct NodeConnection {
        nodes: Option<Vec<Node>>,
    }

    #[derive(Debug, Deserialize)]
    struct Node {
        author: Option<Author>,
        #[serde(rename = "createdAt")]
        created_at: Option<String>,
        #[serde(rename = "closedAt")]
        closed_at: Option<String>,
        state: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    struct Author {
        login: String,
    }

    let mut events: Vec<RawEvent> = Vec::new();

    // Parse the GraphQL response
    let response: GraphQLResponse = match serde_json::from_str(json) {
        Ok(response) => response,
        Err(e) => {
            debug!("Failed to parse GraphQL response: {:?}", e);
            return Err(Error::other(format!(
                "Failed to parse GraphQL response: {e}"
            )));
        }
    };

    // Check for errors in the GraphQL response
    if let Some(errors) = response.errors {
        let error_messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
        debug!("GraphQL errors: {:?}", error_messages);
        return Err(Error::other(format!(
            "GraphQL errors: {}",
            error_messages.join(", ")
        )));
    }

    // Extract data from the response
    if let Some(data) = response.data {
        if let Some(repository) = data.repository {
            // Process issue comments
            if let Some(issue_comments) = repository.issue_comments {
                if let Some(nodes) = issue_comments.nodes {
                    for node in nodes {
                        if let (Some(author), Some(created_at)) = (&node.author, &node.created_at) {
                            let event = RawEvent {
                                actor: Actor {
                                    login: author.login.clone(),
                                },
                                payload: Payload {
                                    action: Action::created,
                                },
                                event_type: Type::IssueCommentEvent,
                                created_at: parse_github_date(created_at)?,
                            };
                            events.push(event);
                        }
                    }
                }
            }

            // Process pull requests
            if let Some(pull_requests) = repository.pull_requests {
                if let Some(nodes) = pull_requests.nodes {
                    for node in nodes {
                        if let (Some(author), Some(created_at)) = (&node.author, &node.created_at) {
                            // Add opened pull request event
                            let opened_event = RawEvent {
                                actor: Actor {
                                    login: author.login.clone(),
                                },
                                payload: Payload {
                                    action: Action::opened,
                                },
                                event_type: Type::PullRequestEvent,
                                created_at: parse_github_date(created_at)?,
                            };
                            events.push(opened_event);

                            // Add closed pull request event if it's closed
                            if let (Some(closed_at), Some(state)) = (&node.closed_at, &node.state) {
                                if state == "CLOSED" || state == "MERGED" {
                                    let closed_event = RawEvent {
                                        actor: Actor {
                                            login: author.login.clone(),
                                        },
                                        payload: Payload {
                                            action: Action::closed,
                                        },
                                        event_type: Type::PullRequestEvent,
                                        created_at: parse_github_date(closed_at)?,
                                    };
                                    events.push(closed_event);
                                }
                            }
                        }
                    }
                }
            }

            // Process pull request reviews
            if let Some(pull_request_reviews) = repository.pull_request_reviews {
                if let Some(nodes) = pull_request_reviews.nodes {
                    for node in nodes {
                        if let (Some(author), Some(created_at)) = (&node.author, &node.created_at) {
                            let event = RawEvent {
                                actor: Actor {
                                    login: author.login.clone(),
                                },
                                payload: Payload {
                                    action: Action::created,
                                },
                                event_type: Type::PullRequestReviewCommentEvent,
                                created_at: parse_github_date(created_at)?,
                            };
                            events.push(event);
                        }
                    }
                }
            }
        } else {
            debug!("Repository not found: {}", repo);
            return Err(Error::other(format!("Repository not found: {repo}")));
        }
    } else {
        debug!("No data in GraphQL response");
        return Err(Error::other("No data in GraphQL response".to_string()));
    }

    Ok(events)
}

/// Parses a date string from GitHub's GraphQL API and converts it to a DateTime<Utc>.
///
/// This function converts an ISO 8601 date string (RFC 3339 format) to a DateTime<Utc> object.
///
/// # Arguments
///
/// * `date_str` - A string slice containing the date in ISO 8601 format
///
/// # Returns
///
/// A Result containing a DateTime<Utc> object or an Error
fn parse_github_date(date_str: &str) -> Result<DateTime<Utc>, Error> {
    match DateTime::parse_from_rfc3339(date_str) {
        Ok(dt) => Ok(dt.with_timezone(&Utc)),
        Err(e) => {
            debug!("Failed to parse date: {:?}", e);
            Err(Error::other(format!("Failed to parse date: {e}")))
        }
    }
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
