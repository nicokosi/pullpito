# pullpito [![Build Status](https://travis-ci.org/nicokosi/pullpito.svg?branch=master)](https://travis-ci.org/nicokosi/pullpito)

Command-line for GitHub pull request statistics 🐙, similar to [hubstats](https://github.com/nicokosi/hubstats).

Implemented while learning [Rust](https://www.rust-lang.org/) 🦀, reading [The Rust Programming Language](https://doc.rust-lang.org/stable/book/second-edition/) 🎓.

## Run

In order to display pull request events for a public GitHub repository, run `cargo run $org/$repo`. Example: `cargo run python/peps`.

For a private GitHub repository, run `cargo run $org/$repo $token`.

## Install

Run `cargo install ~/.cargo/bin/pullpito`.