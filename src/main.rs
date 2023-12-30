use std::env;
use std::io;
use std::process;

fn decimal_matcher(input_line: &str) -> bool {
    for c in input_line.chars() {
        if c.is_digit(10) {
            return true;
        }
    }
    return false;
}

fn word_matcher(input_line: &str) -> bool {
    for c in input_line.chars() {
        if c.is_alphabetic() || c.is_digit(10) {
            return true;
        }
    }
    return false;
}

fn positive_char_group_matcher(input_line: &str, pattern: &str)  -> bool {
    let group = &pattern[1..pattern.len()-1];
    for gc in group.chars() {
        if input_line.contains(gc) {
            println!("{} is in {}", gc, input_line);
            return true;
        }
    }
    return false;
}

fn negative_char_group_matcher(input_line: &str, pattern: &str)  -> bool {
    let group = &pattern[2..pattern.len()-1];
    for c in input_line.trim().chars() {
        if !group.contains(c) {
            println!("{} is not in {}", c, group);
            return true;
        }
    }
    return false;
}

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    if pattern.chars().count() > 0 {
        if pattern == "\\d" {
            println!("Decimal matcher");
            return decimal_matcher(input_line);
        }
        else if pattern == "\\w" {
            println!("word matcher");
            return word_matcher(input_line);
        }
        else if pattern.starts_with("[^") && pattern.ends_with("]") {
            println!("Negative char group matcher");
            return negative_char_group_matcher(input_line, pattern);
        }
        else if pattern.starts_with("[") && pattern.ends_with("]") {
            println!("Positive char group matcher");
            return positive_char_group_matcher(input_line, pattern);
        }
        println!("generic matcher");
        return input_line.contains(pattern);
    } else {
        panic!("Unhandled pattern: {}", pattern)
    }
}

// Usage: echo <input_text> | your_grep.sh -E <pattern>
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    if match_pattern(&input_line, &pattern) {
        println!("Match!");
        process::exit(0)
    } else {
        println!("No match");
        process::exit(1)
    }
}
