# op8d_lexemizer

Opinionated Rust library for transforming code to a vector of Lexemes.

Currently, only Rust edition 2018 is supported.  

* Build the docs: `rm -rf target/doc; cargo doc`
* Read the docs: `open target/doc/op8d_lexemizer/index.html`
* Run the tests: `cargo test`
* Delete cargo’s cache, if new code is being ignored: `cargo clean`
* Try an example: `cargo run --example lexemize-rs2018-arg -- "const FOUR: u8 = 4;"`
