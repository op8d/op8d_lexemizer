use std::{env,process};

use op8d_lexemizer::rust_2018::lexemize::lexemize;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("ERROR: Expected 2 args, got {}. Try:", args.len());
        eprint!("    cargo run --example lexemize-rs2018-arg -- ");
        eprintln!(r#""const ROUGHLY_PI: f32 = 3.14;""#);
        process::exit(1);
    }
    println!("{}", lexemize(&args[1]));
}
