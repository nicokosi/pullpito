# pullpito 🐙 [![Build Status](https://travis-ci.org/nicokosi/pullpito.svg?branch=master)](https://travis-ci.org/nicokosi/pullpito)[![Coverage Status](https://coveralls.io/repos/github/nicokosi/pullpito/badge.svg?branch=master)](https://coveralls.io/github/nicokosi/pullpito?branch=master)

Command-line for GitHub pull request statistics, similar to [hubstats](https://github.com/nicokosi/hubstats).

Implemented while learning [Rust](https://www.rust-lang.org/) 🦀, reading [The Rust Programming Language](https://doc.rust-lang.org/stable/book/second-edition/) 🎓.

## Pre-requisite

Install the [`rustup` Rust toolchain](https://rustup.rs/).

## Run

In order to display pull request events for a **single** public GitHub repository, run `cargo --repository $org/$repo` or `cargo -r $org/$repo`.

For instance, running `cargo run --quiet -- --repository python/peps` will display an output like:

```text
pull requests for "python/peps" ->
  opened per author:
    ssbarnea: 1
    emilyemorehouse: 2
  commented per author:
    brettcannon: 2
  closed per author:
    brettcannon: 2
    gvanrossum: 6
 ```

For a **private** GitHub repository, run `cargo run -- --repository $org/$repo --token $token`.

For **several** GitHub repositories, use several `repository` arguments: `cargo run -- --repository $repo1 --repository $repo2`.

For more information, run `cargo run -- --help`.

### Run with debug logs

```sh
RUST_LOG=pullpito=debug cargo run -- --repository nicokosi/pullpito
```

## Install

Run `cargo install --path .`. You can then run the `pullpito` command directly.

## Development cheat-sheet

- Compile and run tests: `cargo test`
- Format all the code: `cargo fmt`
- Run the linter: `cargo clippy`
