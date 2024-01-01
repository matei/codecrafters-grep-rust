use crate::matcher::Matcher::Association;

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
            match matcher {
                Matcher::Skip => {}
                _ => {
                    matchers.push(matcher);
                }
            }
        }
        Self {matchers, pattern_str: input, start_modifier: start_with, debug}
    }

    pub fn test(&self, input: &str) -> (bool, usize) {
        print_debug(&format!("Start pattern match for {} against {}", self.pattern_str, input), self.debug);
        if self.matchers.len() == 0 {
            return (true, 0);
        }
        let mut start_index = 0;
        while start_index < input.chars().count() {
            let (result, matched_size) = self.do_test(&input[start_index..]);
            if  result {
                return (true, matched_size);
            }
            if self.start_modifier {
                print_debug("No match for entire input and start modifier was requested", self.debug);
                return (false, 0);
            }
            start_index += 1;
            print_debug("", self.debug);
        }
        return (false, 0);
    }

    fn do_test(&self, input: &str) -> (bool, usize) {
        let mut input_pos = 0;
        let mut pattern_pos = 0;

        print_debug(&format!("Testing '{}' against '{}'", input, self.pattern_str), self.debug);

        while input_pos < input.chars().count() && pattern_pos < self.matchers.len() {
            let (result, to_advance) = self.matchers[pattern_pos].test(input, input_pos, self.debug);
            if !result {
                return (false, 0);
            }
            input_pos += to_advance;
            pattern_pos += 1;
        }
        if pattern_pos < self.matchers.len() { // means we finished input without matching the entire pattern
            let mut has_non_wildcards = false;
            while pattern_pos < self.matchers.len() && !has_non_wildcards {
                match &self.matchers[pattern_pos] {
                    Matcher::Skip | Matcher::ZeroOrOne(_) | Matcher::EndOfString  => {
                        has_non_wildcards = true;
                    }
                    _ => ()
                }
                pattern_pos += 1;
            }
            if has_non_wildcards {
                print_debug("Finished input but not pattern", self.debug);
                return (false, 0);
            }
        }
        return (true, input_pos);
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
    OneOrMore(Box<Matcher<'a>>),
    ZeroOrOne(Box<Matcher<'a>>),
    Skip,
    Association(Vec<String>)
}

impl<'a> Matcher<'a> {
    pub fn new(input: &'a str, pos: usize) -> (Self, usize) {
        let mut result: (Matcher, usize); // here is where the language breaks down...
        result = match input.chars().nth(pos) {
            Some('^') => {
                (Self::StartOfString, 0)
            }
            Some('$') => {
                (Self::EndOfString, 0)
            }
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
            }
            Some('+') => {
                (Self::Skip, 0) //handled in lookahead below - normally we shouldn't reach this
            }
            Some('?') => (Self::Skip, 0), //handled in lookahead below - normally we shouldn't reach this
            Some('.') => (Self::WildCard, 0),
            Some('[') => {
                if let Some(closing_bracket) = find_closing(input, ']', pos) {
                    result = match input.chars().nth(pos + 1) {
                        Some('^') => (Self::GroupNegative(&input[pos + 2..closing_bracket]), closing_bracket - pos),
                        Some(_) => (Self::GroupPositive(&input[pos + 1..closing_bracket]), closing_bracket - pos),
                        None => panic!("This cannot happen")
                    };
                }
                else {
                    panic!("Unclosed [");
                }
                result
            }
            Some('(') => {
                if let Some(closing_p) = find_closing(input, ')', pos) {
                    result = (Association(input[pos + 1..closing_p].split("|").map(|s| s.to_string()).collect()), closing_p - pos);
                } else {
                    panic!("Unclosed (");
                }
                result
            }
            Some(']') => (Self::Skip, 0),
            Some(')') => (Self::Skip, 0),
            Some(c) => (Self::Literal(c), 0),
            None => panic!("Could not compile pattern"),
        };

        //lookahead for quantifiers
        let (matcher, skip) = result;
        match input.chars().nth(pos + skip + 1) {
            Some('+') => {
                return (Self::OneOrMore(Box::new(matcher)), skip + 1);
            },
            Some('?') => {
                return (Self::ZeroOrOne(Box::new(matcher)), skip + 1);
            }
            Some(_) => (),
            None => ()
        }

        return (matcher, skip);
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
            }
            Self::Digit => {
                match input.chars().nth(pos) {
                    Some(c) => {
                        let result = c.is_digit(10);
                        print_debug(&format!("Match {} \\d with {}", result, c), debug);
                        (result, 1)
                    },
                    None => (false, 0)
                }
            }
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
            }
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
            }
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
            }
            Self::EndOfString => {
                // cats cat$
                let result = pos >= input.chars().count();
                print_debug(&format!("Match {} $ at position {}", result, pos), debug);
                (result, 1)
            }
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
            }
            Self::OneOrMore(pc) => {
                let mut matched = 0;
                return match input.chars().nth(pos) {
                    Some(_) => {
                        let (result, advance) = pc.test(&input, pos, debug);
                        if result {
                            matched += advance;
                            while pos + matched < input.chars().count() {
                                let (result, advance) = pc.test(&input, pos + matched, debug);
                                if result {
                                    matched += advance;
                                }
                                else {
                                    break;
                                }
                            }
                        }
                        return if matched == 0 {
                            print_debug(&format!("Match false {:?}+", *pc),  debug);
                            (false, 1)
                        } else {
                            print_debug(&format!("Match true {:?}+ {} times", *pc, matched), debug);
                            (true, matched)
                        }
                    },
                    None => {
                        print_debug(&format!("No match for {:?}+ at position {}", *pc, pos), debug);
                        (false, 0)
                    }
                }
            }
            Self::ZeroOrOne(pc) => {
                //doa?g - dog
                match input.chars().nth(pos) {
                    Some(c) => {
                        let (result, advance) = pc.test(&input, pos, debug);
                        if result {
                            print_debug(&format!("Optional match true {:?}? with {} ", *pc, c), debug);
                            (true, advance)
                        }
                        else {
                            print_debug(&format!("Optional match false {:?}? with {} ", *pc, c), debug);
                            (true, 0)
                        }
                    },
                    None => {
                        print_debug(&format!("Optional match false {:?}? at end of input ", *pc), debug);
                        (true, 0)
                    }
                }
            }
            Self::Association(parts) => {
                for sub_pattern in parts {
                    let sub_input = &input[pos..].to_string();
                    let sub_pattern_matcher = Pattern::new(sub_pattern, debug);
                    let (result, to_advance) = sub_pattern_matcher.test(&sub_input);
                    if result {
                        print_debug(&format!("Match true subpattern {} from association group with {} characters at position {}", sub_pattern, to_advance, pos), debug);
                        return (true, to_advance);
                    } else {
                        print_debug(&format!("Match false subpattern {} from association group at position {}", sub_pattern, pos), debug);
                    }
                }
                print_debug(&format!("Match false entire association group at position {}",  pos), debug);
                (false, 1)
            }
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