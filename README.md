# pullpito [![Build Status](https://travis-ci.org/nicokosi/pullpito.svg?branch=master)](https://travis-ci.org/nicokosi/pullpito)

Command-line for GitHub pull request statistics 🐙, similar to [hubstats](https://github.com/nicokosi/hubstats).

Implemented while learning [Rust](https://www.rust-lang.org/) 🦀, reading [The Rust Programming Language](https://doc.rust-lang.org/stable/book/second-edition/) 🎓.


## Run

In order to display pull request events for a public GitHub repository, run `cargo run $org/$repo`.

Example:

```sh
cargo run --quiet python/peps

pull requests for "python/peps" ->
     opened per author: { "Rosuav" 0 "pitrou" 0 "ncoghlan" 1 "jseakle" 1 "yoavcaspi" 1  }
     commented per author: { "Rosuav" 1 "pitrou" 1 "ncoghlan" 1 "jseakle" 0 "yoavcaspi" 0  }
     closed per author: { "Rosuav" 0 "pitrou" 0 "ncoghlan" 0 "jseakle" 0 "yoavcaspi" 0 }
```

For a private GitHub repository, run `cargo run $org/$repo $token`.


## Install

Run `cargo install ~/.cargo/bin/pullpito`.