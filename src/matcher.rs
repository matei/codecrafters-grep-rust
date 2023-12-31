#[derive(Debug)]
pub struct Pattern<'a> {
    matchers: Vec<Matcher<'a>>,
    start_modifier: bool,
    pattern_str: &'a str,
    debug: bool
}

impl<'a> Pattern<'a> {
    pub fn new(input: &'a str, debug: bool) -> Self {
        let mut matchers: Vec<Matcher> = Vec::new();
        let mut skip = 0;
        let start_with = input.chars().nth(0).unwrap_or('a') == '^';

        for i in 0..input.chars().count() {
            if skip > 0 {
                skip -= 1;
                continue;
            }
            if i == 0 && start_with {
                continue;
            }
            let (matcher, skip_s) = Matcher::new(&input, i);
            skip = skip_s;
            matchers.push(matcher);
        }
        Self {matchers, pattern_str: input, start_modifier: start_with, debug}
    }

    pub fn test(&self, input: &str) -> bool {
        print_debug(&format!("Start pattern match for {} against {}", self.pattern_str, input), self.debug);
        if self.matchers.len() == 0 {
            return true;
        }
        let mut start_index = 0;
        while start_index < input.chars().count() {
            if self.do_test(&input[start_index..]) {
                return true;
            }
            if self.start_modifier {
                print_debug("No match for entire input and start modifier was requested", self.debug);
                return false;
            }
            start_index += 1;
            print_debug("", self.debug);
        }
        return false;
    }

    fn do_test(&self, input: &str) -> bool {
        let mut input_pos = 0;
        let mut pattern_pos = 0;

        print_debug(&format!("Testing {} against {}", input, self.pattern_str), self.debug);

        while input_pos < input.chars().count() && pattern_pos < self.matchers.len() {
            let (result, to_advance) = self.matchers[pattern_pos].test(input, input_pos, self.debug);
            if !result {
                return false;
            }
            input_pos += to_advance;
            pattern_pos += 1;
        }
        if pattern_pos < self.matchers.len() { // means we finished input without matching the entire pattern
            let mut has_non_wildcards = false;
            while pattern_pos < self.matchers.len() && !has_non_wildcards {
                if ![Matcher::Skip, Matcher::ZeroOrOne(input.chars().last().unwrap()), Matcher::EndOfString].contains(&self.matchers[pattern_pos]) {
                    has_non_wildcards = true;
                }
                pattern_pos += 1;
            }
            if has_non_wildcards {
                print_debug("Finished input but not pattern", self.debug);
                return false;
            }
        }
        return true;
    }
}
#[derive(Debug, PartialEq)]
enum Matcher<'a> {
    Literal(char),
    Digit,
    Word,
    GroupPositive(&'a str),
    GroupNegative(&'a str),
    StartOfString,
    EndOfString,
    WildCard,
    OneOrMore(char),
    ZeroOrOne(char),
    Skip
}

impl<'a> Matcher<'a> {
    pub fn new(input: &'a str, pos: usize) -> (Self, usize) {
        match input.chars().nth(pos) {
            Some('^') => {
                (Self::StartOfString, 0)
            },
            Some('$') => {
                (Self::EndOfString, 0)
            },
            Some('\\') => {
                match input.chars().nth(pos + 1) {
                    Some('d') => {
                        (Self::Digit, 1)
                    },
                    Some('w') => {
                        (Self::Word, 1)
                    },
                    Some('\\') => {
                        (Self::Literal('\\'), 1)
                    },
                    Some(_) => {
                        panic!("Unrecognized escape sequence")
                    },
                    None => {
                        panic!("Unterminated escape sequence")
                    }
                }
            },
            Some('+') => {
                (Self::Skip, 0) //handled in generic Some(c) - normally we shouldn't reach this
            },
            Some('?') => (Self::Skip, 0), //handled in generic Some(c) - normally we shouldn't reach this
            Some('.') => (Self::WildCard, 0),
            Some('[') => {
                if let Some(closing_bracket) = find_closing(input, ']', pos) {
                    match input.chars().nth(pos + 1) {
                        Some('^') => (Self::GroupNegative(&input[pos + 2..closing_bracket]), closing_bracket - pos),
                        Some(_) => (Self::GroupPositive(&input[pos + 1..closing_bracket]), closing_bracket - pos),
                        None => panic!("This cannot happen")
                    }
                }
                else {
                    panic!("Unclosed [");
                }
            },
            Some(']') => (Self::Skip, 0),
            Some(c) => {
                match input.chars().nth(pos + 1) {
                    Some('+') => (Self::OneOrMore(c), 1),
                    Some('?') => (Self::ZeroOrOne(c), 1),
                    Some(_) => (Self::Literal(c), 0),
                    None => (Self::Literal(c), 0)
                }
            },
            None => panic!("Could not compile pattern")
        }
    }

    fn test(&self, input: &str, pos: usize, debug: bool) -> (bool, usize) {
        match self {
            Self::Literal(pc) => {
                match input.chars().nth(pos) {
                    Some(c) => {
                        let result = pc == &c;
                        print_debug(&format!("Match {} {} with {}", result, pc, c), debug);
                        (result, 1)
                    },
                    None => (false, 0)
                }
            },
            Self::Digit => {
                match input.chars().nth(pos) {
                    Some(c) => {
                        let result = c.is_digit(10);
                        print_debug(&format!("Match {} \\d with {}", result, c), debug);
                        (result, 1)
                    },
                    None => (false, 0)
                }
            },
            Self::Word => {
                match input.chars().nth(pos) {
                    Some(c) => {
                        let result = c.is_alphabetic() || c.is_digit(10);
                        print_debug(&format!("Match {} \\w with {}", result, c), debug);
                        (result, 1)
                    },
                    None => {
                        print_debug(&format!("No match for \\w at position {}",  pos), debug);
                        (false, 0)
                    }
                }
            },
            Self::GroupPositive(group) => {
                match input.chars().nth(pos) {
                    Some(c) => {
                        let result = group.contains(c);
                        print_debug(&format!("Match {} [{}] with {}", result, group, c), debug);
                        (result, 1)
                    },
                    None => {
                        print_debug(&format!("No match for [{}] at position {}", group, pos), debug);
                        (false, 0)
                    }
                }
            },
            Self::GroupNegative(group) => {
                match input.chars().nth(pos) {
                    Some(c) => {
                        let result = !group.contains(c);
                        print_debug(&format!("Match {} [^{}] with {}", result, group, c), debug);
                        (result, 1)
                    },
                    None => {
                        print_debug(&format!("No match for [^{}] at position {}", group, pos), debug);
                        (false, 0)
                    }
                }
            },
            Self::EndOfString => {
                // cats cat$
                let result = pos >= input.chars().count();
                print_debug(&format!("Match {} $ at position {}", result, pos), debug);
                (result, 1)
            },
            Self::WildCard => {
                match input.chars().nth(pos) {
                    Some(c) => {
                        print_debug(&format!("Matched . with {}", c), debug);
                        (true, 1)
                    },
                    None => {
                        print_debug(&format!("No match for . at position {}", pos), debug);
                        (false, 0)
                    }
                }
            },
            Self::OneOrMore(pc) => {
                let mut matched = 0;
                return match input.chars().nth(pos) {
                    Some(c) => {
                        if &c != pc {
                            print_debug(&format!("Match false for {}+ with {}", pc, c), debug);
                            return (false, 1)
                        }
                        let mut search_pos = pos;
                        while search_pos < input.chars().count() && &input.chars().nth(search_pos).unwrap() == pc {
                            search_pos += 1;
                            matched += 1;
                        }
                        print_debug(&format!("Match true {}+ with {}x{}", pc, matched, pc), debug);
                        (true, matched)
                    },
                    None => {
                        print_debug(&format!("No match for {}+ at position {}", pc, pos), debug);
                        (false, 0)
                    }
                }
            },
            Self::ZeroOrOne(pc) => {
                //doa?g - dog
                match input.chars().nth(pos) {
                    Some(c) => {
                        if pc == &c {
                            print_debug(&format!("Optional match true {}? with {} ", pc, c), debug);
                            (true, 1)
                        }
                        else {
                            print_debug(&format!("Optional match false {}? with {} ", pc, c), debug);
                            (true, 0)
                        }
                    },
                    None => {
                        print_debug(&format!("Optional match false {}? at end of input ", pc), debug);
                        (true, 0)
                    }
                }
            },
            Self::Skip => {
                print_debug(&format!("Skip at position {}", pos), debug);
                (true, 0)
            },
            _ => panic!("Invalid pattern")
        }
    }
}

fn find_closing(input: &str, element: char, after: usize) -> Option<usize> {
    for (i, _) in input.match_indices(element) {
        if i > after {
            return Some(i);
        }
    }
    return None;
}

fn print_debug(message: &str, debug: bool) {
    if debug {
        println!("{}", message);
    }
}