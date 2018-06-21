# pullpito [![Build Status](https://travis-ci.org/nicokosi/pullpito.svg?branch=master)](https://travis-ci.org/nicokosi/pullpito)

Command-line for GitHub pull request statistics 🐙, similar to [hubstats](https://github.com/nicokosi/hubstats).

Implemented while learning [Rust](https://www.rust-lang.org/) 🦀, reading [The Rust Programming Language](https://doc.rust-lang.org/stable/book/second-edition/) 🎓.


## Run

In order to display pull request events for a *single public GitHub repository*, run `cargo run $org/$repo`.

For instance, running `cargo run --quiet python/peps` will display:
```
pull requests for "python/peps" ->
  opened per author:
    ilevkivskyi: 1
    zhsj: 1
    jdemeyer: 2
    egaudry: 1
  commented per author:
    the-knights-who-say-ni: 1
    Rosuav: 1
  closed per author:
    ilevkivskyi: 1
    methane: 1
    Rosuav: 3
```

For a *private GitHub repository*, run `cargo run $org/$repo $token`.

For *several GitHub repositories*, use a comma-separated list:  `cargo $repo1,$repo2 $org/$repo $token`

### Run with debug logs

```sh
RUST_LOG=pullpito=debug cargo run nicokosi/pullpito
```


## Install

Run `cargo install ~/.cargo/bin/pullpito`. You can then run the `pullpito` command directly.