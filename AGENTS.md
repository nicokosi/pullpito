# AGENTS.md

Instructions for AI coding agents working on pullpito.

## Development Environment

- Rust toolchain via `rustup` (see README.md for installation)
- Project uses Rust 2024 edition

## Build and Test Commands

```bash
# Run tests
cargo test

# Format code
cargo fmt

# Run linter
cargo clippy

# Build and run
cargo run -- --repository <org/repo>

# Build release binary
cargo build --release
```

## Project Structure

- `src/main.rs` - Entry point
- `src/lib.rs` - Main library with CLI parsing and statistics logic
- `src/github_events/mod.rs` - GitHub API interaction module
- `test/` - JSON fixtures for tests

## Key Implementation Details

### GitHub API Integration
- Fetches events from GitHub REST API (`/repos/{repo}/events`)
- Supports pagination via Link headers
- Handles authentication via optional token
- Uses blocking HTTP client (reqwest)

### Event Processing
- Filters for: `PullRequestEvent`, `PullRequestReviewCommentEvent`, `IssueCommentEvent`
- Groups events by actor/author
- Tracks actions: `opened`, `created`, `closed`

### Concurrency
- Multi-repository queries run in parallel using threads and channels
- Each repository is processed in its own thread

## Coding Conventions

- Use `#[allow(non_camel_case_types)]` for enum variants matching GitHub API response format
- Deserialize with custom handlers for enum fields to handle unknown variants gracefully
- Prefer `unwrap()` for expected operations (API failures are acceptable panics per doc comments)
- Use `lazy_static!` for compiled regex patterns
- Format strings with `{var:?}` for debug output, `{var}` for display
- Keep test fixtures in `test/` directory with descriptive names

## Testing Practices

- Unit tests in same file as implementation using `#[cfg(test)] mod tests`
- Test fixtures stored as JSON files in `test/` directory
- Use `include_str!()` macro to load test fixtures
- Test both happy paths and edge cases (unknown enums, pagination, etc.)

## Dependencies

Key dependencies to be aware of:
- `structopt` - CLI argument parsing
- `serde`/`serde_json` - JSON serialization
- `reqwest` - HTTP client (blocking mode)
- `chrono` - DateTime handling
- `log`/`env_logger` - Logging infrastructure

## Logging

- Use `log` crate macros: `info!`, `debug!`, `trace!`
- Controlled via `RUST_LOG` environment variable
- `trace` level may contain sensitive data (tokens)
