use std::env;
use std::io;
use std::process;

struct Pattern<'a> {
    pattern: &'a str,
    pattern_pos: usize,
    input_pos: usize
}

impl Pattern<'_> {
    fn advance(&mut self, input_line: &str, match_end: bool) -> bool {
        self.pattern_pos += 1;
        self.input_pos += 1;
        return self.match_string(input_line, match_end);
    }

    fn reset(&mut self) {
        self.input_pos = 0;
        self.pattern_pos = 0;
    }

    fn match_string(&mut self, input_line: &str, match_end: bool) -> bool {
        //check for $ condition
        if match_end && self.input_pos < input_line.chars().count() && self.pattern_pos >= self.pattern.chars().count() {
            return false;
        }

        if self.pattern_pos >= self.pattern.chars().count() {
            return true;
        }
        if self.input_pos >= input_line.chars().count() && self.pattern_pos < self.pattern.chars().count() {
            //edge case: if last parts of pattern are just "*" or "?" then it's match
            let mut non_wildcards = false;
            let mut pos = self.pattern_pos;
            while pos < self.pattern.chars().count() && !non_wildcards {
                if !['?', '*'].contains(&self.pattern.chars().nth(pos).unwrap()) {
                    non_wildcards = true;
                }
                pos += 1;
            }
            println!("Reached end of input ({}) and not end of pattern ({}). Remaining all wildcards: {}", self.input_pos, self.pattern_pos, !non_wildcards);
            return !non_wildcards;
        }

        let pattern_c = self.pattern.chars().nth(self.pattern_pos).unwrap();
        let input_c = input_line.chars().nth(self.input_pos).unwrap();

        if pattern_c == '\\' {
            if self.pattern.chars().count() - 1 > self.pattern_pos {
                match self.pattern.chars().nth(self.pattern_pos + 1).unwrap() {
                    'd' => {
                        if input_c.is_digit(10) {
                            self.pattern_pos += 1;
                            println!("Match {} with \\d", input_c);
                            return self.advance(input_line, match_end);
                        }
                        println!("Character at position {} ({}) is not a digit", self.input_pos, input_c);
                        return false;
                    },
                    'w' => {
                        if input_c.is_digit(10) || input_c.is_alphabetic() {
                            self.pattern_pos += 1;
                            println!("Match {} with \\w", input_c);
                            return self.advance(input_line, match_end);
                        }
                        println!("Character at position {} ({}) is not a word", self.input_pos, input_c);
                        return false;
                    },
                    '\\' => {
                        if input_c != '\\' {
                            return false;
                        }
                        println!("Match {} with \\", input_c);
                        self.pattern_pos += 1;
                        return self.advance(input_line, match_end);
                    }
                    _ => {
                        panic!("Unhandled escape sequence \\{} in {}", self.pattern.chars().nth(self.pattern_pos + 1).unwrap(), self.pattern);
                    }
                }
            } else {
                panic!("Unterminated escape sequence: {}", self.pattern);
            }
        } else if pattern_c == '[' {
            let mut group = String::new();
            let mut closing_bracket = -1;
            let mut is_negative_group = false;
            let mut pos = self.pattern_pos + 1;
            while pos < self.pattern.chars().count() && closing_bracket == -1 {
                let gc = self.pattern.chars().nth(pos).unwrap();
                if gc == ']' {
                    closing_bracket = pos as i32;
                } else if pos == self.pattern_pos + 1 && gc == '^' {
                    is_negative_group = true;
                } else {
                    group.push(gc);
                }
                pos += 1;
            }
            if closing_bracket == -1 {
                panic!("Unterminated group sequence in {}", self.pattern);
            }
            return if (is_negative_group && group.contains(input_c)) || (!is_negative_group && !group.contains(input_c)) {
                println!("{}[{}] is not matching group {} - is_negative={}", input_c, self.input_pos, group, is_negative_group);
                false
            } else {
                self.pattern_pos = closing_bracket as usize;
                self.advance(input_line, match_end)
            }
        } else if pattern_c == '+' {
            let prev_char = self.pattern.chars().nth(self.pattern_pos - 1).unwrap();
            println!("+ detected, trying to match with {} input_pos is {}", prev_char, self.input_pos);
            while input_line.chars().nth(self.input_pos).unwrap() == prev_char && self.input_pos < input_line.chars().count() - 1 {
                println!("next -> input_pos is {}", self.input_pos);
                self.input_pos += 1;
            }
            self.input_pos -= 1;
            return self.advance(input_line, match_end);
        } else if pattern_c == input_c {
            println!("Match {}[{}] with {}[{}]", pattern_c, self.pattern_pos, input_c, self.input_pos);
            return self.advance(input_line, match_end);
        } else if pattern_c != input_c {
            if pattern_c == '?' {
                //if we are here, the prev pattern char - the optional - was matched, so we just ignore and advance
                println!("Detected ? with prev match, just advance");
                self.input_pos -= 1; //hold the input, skip just the pattern char
                return self.advance(input_line, match_end);
            }
            else if self.pattern_pos < self.pattern.chars().count() - 1 && self.pattern.chars().nth(self.pattern_pos + 1).unwrap() == '?' {
                // for the case abc? -> abdv where the prev optional was not matched
                println!("Detected ? with non-prev match, skip & advance");
                self.pattern_pos += 1; //skip the unmatched optional char and the "?" but the additional increment is in advance
                self.input_pos -= 1; //hold the input, skip just the pattern char
                return self.advance(input_line, match_end);
            }
            else {
                println!("No match {}[{}] with {}[{}]", self.pattern.chars().nth(self.pattern_pos).unwrap(), self.pattern_pos, input_line.chars().nth(self.input_pos).unwrap(), self.input_pos);
                return false;
            }
        }
        //if we reached here, it's the end of the line and there was no return false so it's a match
        println!("Reached end, input_pos={} pattern_pos={}", self.input_pos, self.pattern_pos);
        return true;
    }
}

fn match_pattern(input_line: &str, pattern_str: &str) -> bool {
    if pattern_str.chars().count() > 0 {
        let mut match_start = false;
        let mut match_end = false;
        let mut final_pattern = pattern_str.to_string();
        if pattern_str.chars().nth(0).unwrap() == '^' {
            match_start = true;
            final_pattern = final_pattern[1..].to_string();
        }
        if pattern_str.chars().last().unwrap() == '$' {
            match_end = true;
            final_pattern = final_pattern[..final_pattern.chars().count()-1].to_string();
        }

        let mut pattern = Pattern {
            pattern: &final_pattern,
            pattern_pos: 0,
            input_pos: 0
        };
        let mut start = 0;
        let mut matched = false;

        while start < input_line.chars().count() && ! matched {
            let input = &input_line.to_string()[start..];
            pattern.reset();
            println!();
            println!("Trying to match string '{}' with pattern '{}'", input, pattern_str);
            matched = pattern.match_string(input, match_end);
            if match_start {
                return matched;
            }
            start += 1;
        }
        return matched;
    } else {
        panic!("Unhandled pattern: {}", pattern_str);
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

    let pattern = String::from(env::args().nth(2).unwrap().trim());
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    if match_pattern(&input_line.trim(), &pattern) {
        println!("Match!");
        process::exit(0)
    } else {
        println!("No match");
        process::exit(1)
    }
}
