# pullpito [![Build Status](https://travis-ci.org/nicokosi/pullpito.svg?branch=master)](https://travis-ci.org/nicokosi/pullpito)

Command-line for GitHub pull request statistics 🐙, similar to [hubstats](https://github.com/nicokosi/hubstats).

Implemented while learning [Rust](https://www.rust-lang.org/) 🦀, reading [The Rust Programming Language](https://doc.rust-lang.org/stable/book/second-edition/) 🎓.


## Run

In order to display pull request events for a public GitHub repository, run `cargo run $org/$repo`.

Example:

```sh
pull requests for "python/peps" ->
  opened per author: {
    Harmon758: 2
  }
  commented per author: {
    gvanrossum: 1
    zware: 2
    mcepl: 1
    ned-deily: 1
    warsaw: 1
    encukou: 2
  }
  closed per author: {
    zware: 2
  }
```

For a private GitHub repository, run `cargo run $org/$repo $token`.


## Install

Run `cargo install ~/.cargo/bin/pullpito`.