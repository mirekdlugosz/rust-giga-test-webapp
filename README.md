# Giga Test webapp

Giga Test was a competition organized by _PSX Extreme_, a Polish video game magazine. Original contest took place between September 2000 and February 2001.

This repository contains a digital copy of the test. The core file is `resources/gigatest.toml`, which contains all the questions, answers, introductions etc. As far as I can tell, this is the only copy in structured data format.

The rest of repository is a web application written in Rust. Once started, you can take a test in the comfort of your web browser.

## Installation

Use `cargo` to fetch all dependencies and build a binary:

    cargo build --release --locked

## Usage

After running `cargo build`, the binary can be found at `target/release/rust-giga-test-webapp`. Run it to start a server. `Ctrl+C` closes it.

The binary recognizes few environment variables, defined in `.env.sample`. They are all optional and their usage should be self-explanatory.
