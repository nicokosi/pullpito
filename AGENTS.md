# AGENTS.md

This file provides guidance to agents when working with code in this repository.

## Project Overview

Pullpito is a command-line tool for GitHub pull request statistics, implemented in Rust as a learning project. It fetches GitHub API events and analyzes pull request activities (opened, commented, closed) per author for one or more repositories.

## Development Commands

### Core Development Tasks
- **Build and run**: `cargo run -- --repository org/repo`
- **Run tests**: `cargo test`
- **Format code**: `cargo fmt`
- **Lint code**: `cargo clippy`
- **Check formatting**: `cargo fmt -- --check`
- **Lint with warnings as errors**: `cargo clippy -- -Dwarnings`

### Running the Application
- **Single public repo**: `cargo run -- --repository python/peps`
- **Private repo with token**: `cargo run -- --repository org/repo --token $GITHUB_TOKEN`
- **Multiple repos**: `cargo run -- --repository repo1 --repository repo2`
- **With debug logs**: `RUST_LOG=pullpito=debug cargo run -- --repository org/repo`
- **With trace logs (includes sensitive data)**: `RUST_LOG=pullpito=trace cargo run -- --repository org/repo`

### Testing
- **Run all tests**: `cargo test`
- **Run specific test**: `cargo test test_name`
- **Run tests with output**: `cargo test -- --nocapture`

### Installation
- **Install locally**: `cargo install --path .`

## Architecture Overview

### High-Level Structure
The application follows a simple but well-structured architecture:

1. **Main Entry Point** (`src/main.rs`): Minimal entry point that delegates to the library
2. **Core Library** (`src/lib.rs`): Contains the main application logic, CLI argument parsing, and orchestration
3. **GitHub Events Module** (`src/github_events/mod.rs`): Handles GitHub API communication and data models

### Key Components

#### Command Line Interface
- Uses `structopt` for argument parsing
- Supports multiple repositories via repeated `--repository` flags
- Optional GitHub token for private repositories
- Short (`-r`, `-t`) and long (`--repository`, `--token`) argument forms

#### Data Flow Architecture
1. **Configuration**: CLI args parsed into `Config` struct
2. **Concurrent Processing**: Each repository is processed in a separate thread using `mpsc` channels
3. **API Integration**: GitHub Events API is called with pagination support
4. **Data Transformation**: Raw events are filtered and grouped by author
5. **Output Generation**: Statistics are formatted and printed for each repository

#### GitHub API Integration
- **Pagination Handling**: Automatically follows GitHub API pagination using Link headers
- **Authentication**: Supports optional GitHub personal access tokens
- **Error Handling**: Graceful handling of API limits and authentication issues
- **Event Filtering**: Focuses on pull request events, review comments, and issue comments
- **Rate Limiting**: Respects GitHub API rate limits through proper error handling

#### Data Models
- **RawEvent**: Represents GitHub webhook events with actor, payload, type, and timestamp
- **Actor**: GitHub user information (login)
- **Payload**: Event action (opened, closed, created, or Unknown)
- **Type**: Event types (PullRequestEvent, IssueCommentEvent, PullRequestReviewCommentEvent, Unknown)

### Threading and Concurrency
The application uses Rust's `std::sync::mpsc` for concurrent processing:
- Main thread spawns worker threads for each repository
- Each worker thread makes GitHub API calls independently
- Results are collected via message passing and printed in order received

### Error Handling Strategy
- Uses Rust's `Result<T, E>` pattern throughout
- Graceful degradation for API errors (rate limits, authentication)
- Unknown enum variants are handled via `Unknown` fallback values
- Comprehensive logging at debug and trace levels

### Testing Approach
- Unit tests for CLI argument parsing
- Integration tests with real GitHub API response data (stored in `test/` directory)
- Tests cover both successful parsing and edge cases (unknown enum values)
- Mock data includes real responses from `python/peps` and `nicokosi/pullpito` repositories

## Important Implementation Details

### GitHub API Specifics
- Uses GitHub Events API (`/repos/{owner}/{repo}/events`)
- Processes up to 10 pages of events (configurable in code)
- User-Agent header required: `nicokosi/pullpito`
- Token authentication format: `token {personal_access_token}`

### Deserialization Patterns
- Custom deserializers handle unknown enum variants gracefully
- Fields default to `Unknown` variants when GitHub API returns unexpected values
- DateTime parsing uses `chrono` with UTC timezone

### Logging Levels
- **INFO**: Repository processing status
- **DEBUG**: API pagination, content availability, response headers
- **TRACE**: Full API request/response details including sensitive token information
