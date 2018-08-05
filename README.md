# sublercli-rs

[![Build Status](https://travis-ci.com/MattsSe/rust-subler.svg?branch=master)](https://travis-ci.com/MattsSe/rust-subler)
[![Released API docs](https://docs.rs/sublercli/badge.svg)](https://docs.rs/sublercli)

A simple commandline interface for the sublerCLI tool on mac OSto write metadata to media files

## Installation

Requires an additional [SublerCLI](https://bitbucket.org/galad87/sublercli) Installation.
To install with homebrew: `brew cask install sublercli`

By default `sublercli-rs` assumes a `homebrew` installation under `/usr/local/bin/SublerCli`
You can check your installtion path with `brew cask info sublercli`
If the SublerCLI installation destination deviates from default, you can overwerite the path
by setting the `SUBLER_CLI_PATH` environment variable to the valid destination.

## Atoms

To store metadata, Atoms are used. An Atom has a specifc name and the value it stores.
The `Atom` struct mimics this behavior. There is a predefined set of valid atoms.
To obtain a list of al valid metadata atom tag names:

 ```rust
 use sublercli::Atoms;
 let valid_tags: Vec<&str> = Atoms::metadata_tags();
 ```

 Support for the predefined set of known atoms is individually implemented. `Atoms` functions as a wrapper to store a set of single `Atom` values and is used to create Atoms like:

```rust
use sublercli::*;  
let atoms = Atoms::new()
    .add("Cast", "John Doe")
    .genre("Foo,Bar")
    .artist("Foo Artist")
    .title("Foo Bar Title")
    .release_date("2018")
    .build();
 ```

## Tagging

To invoke the SublerCLI process:
If no dest path is supplied then the destination path is the existing file name suffixed, starting from 0: `demo.mp4 -> demo.0.mp4`

```rust
use sublercli::*;
let file = "demo.mp4";
let subler = Subler::new(file, Atoms::new().title("Foo Bar Title").build())
    // by default, mediakind is already set to `Movie`
    .media_kind(Some(MediaKind::Movie))

    // set an optional destination path
    .dest("dest/path")

    // by default the optimization flag is set to true
    .optimize(false)

    // execute prcess in sync,
    // alternativly spawn the process: `.spawn_tag()`
    .tag()

    .and_then(|x| {
        println!("stdout: {}", String::from_utf8_lossy(&x.stdout));
        Ok(())
    });
 ```