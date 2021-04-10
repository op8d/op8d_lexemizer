use std::{env,fs,process};

use op8d_lexemizer::rust_2018::lexemize::lexemize;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("ERROR: Expected 2 args, got {}. Try:", args.len());
        eprintln!(r#"    echo "const FOUR: u8 = 4;" > four.rs"#);
        eprintln!("    cargo run --example lexemize-rs2018-file -- four.rs");
        process::exit(1);
    }
    let contents = fs::read_to_string(&args[1]).unwrap_or_else(|err| {
        eprintln!("ERROR: Problem reading the file:\n    {}", err);
        process::exit(2);
    });
    // See stackoverflow.com/a/60581271 and reddit.com/r/rust/comments/cfybfa
    println!("{}", lexemize(Box::leak(contents.into_boxed_str())));
}