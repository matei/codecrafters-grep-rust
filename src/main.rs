use std::env;
use std::io;
use std::process;
use crate::matcher::Pattern;
extern crate pretty_env_logger;
#[macro_use] extern crate log;

mod matcher;

// Usage: echo <input_text> | your_grep.sh -E <pattern>
fn main() {
    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }
    let debug = env::args().count() == 4 && env::args().nth(3).unwrap() == "-d";
    if debug {
        std::env::set_var("RUST_LOG", "trace");
    }
    pretty_env_logger::init();
    let pattern = String::from(env::args().nth(2).unwrap().trim());
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    let mut pattern_matcher = Pattern::new(&pattern, debug);

    // dbg!(&pattern_matcher);
    info!("{:?}", &pattern_matcher);

    let (result, _) = pattern_matcher.test(&input_line.trim());
    if result {
        println!("Match!");
        process::exit(0)
    } else {
        println!("No match");
        process::exit(1)
    }
}
