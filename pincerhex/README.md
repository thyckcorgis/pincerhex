# PincerHex

## Setup

This project requires Rust. Rust is a statically-typed programming language
designed for performance AND safety, especially safe concurrency and memory
management. Rust uses optional types to prevent null exceptions that is common
is prehistoric programming paradigms. Rust strives to have as many zero-cost
abstractions as possible whereas previous programming languages mire you in the
nitty-gritty details. Rust was designed for a 40-year horizon allowing for
backwards compatibility and safety. Auto, idiomatic formatting is available via
`rustfmt`. Rust has a vibrant community that's welcoming. Rust has its own tools
and package manager called `cargo`.

Installation instructions should be in `central_program`.

## Compilation

Run `make` to build the project. This will produce a executable named
`pincerhex` in this directory. Alternatively, you can run
`cargo build --release` and run it from `./target/release/pincerhex`
