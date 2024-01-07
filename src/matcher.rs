use std::collections::HashMap;
use crate::matcher::Matcher::Association;

#[derive(Debug)]
pub struct Pattern<'a> {
    matchers: Vec<Matcher<'a>>,
    backrefs: HashMap<u32, String>,
    start_modifier: bool,
    pattern_str: &'a str,
    debug: bool
}

impl<'a> Pattern<'a> {
    pub fn new(input: &'a str, debug: bool) -> Self {
        let mut matchers: Vec<Matcher> = Vec::new();
        let mut backrefs: HashMap<u32, String> = HashMap::new();
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
            // print_debug(&format!("instance {:?} skip_s is {}", matcher, skip_s), debug);
            skip = skip_s;
            match matcher {
                Matcher::Skip => {}
                _ => {
                    matchers.push(matcher);
                }
            }
        }
        Self {matchers, backrefs, pattern_str: input, start_modifier: start_with, debug}
    }

    pub fn test(&mut self, input: &str) -> (bool, usize) {
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

    fn do_test(&mut self, input: &str) -> (bool, usize) {
        let mut input_pos = 0;
        let mut pattern_pos = 0;
        self.backrefs.clear();
        let mut backref_count = 1;

        print_debug(&format!("Testing '{}' against '{}'", input, self.pattern_str), self.debug);

        while input_pos < input.chars().count() && pattern_pos < self.matchers.len() {
            let (result, to_advance) = self.matchers[pattern_pos].test(input, input_pos, &self.backrefs, self.debug);
            if !result {
                return (false, 0);
            }
            match self.matchers[pattern_pos] {
                Matcher::Association(_) => {
                    self.backrefs.insert(backref_count, String::from(&input[input_pos..input_pos + to_advance]));
                    backref_count += 1;
                },
                _ => {}
            };
            input_pos += to_advance;
            pattern_pos += 1;
        }
        if pattern_pos < self.matchers.len() { // means we finished input without matching the entire pattern
            print_debug("Finished pattern early, checking for ending wildcards...", self.debug);
            let mut has_non_wildcards = false;
            while pattern_pos < self.matchers.len() && !has_non_wildcards {
                print_debug(&format!("Check {:?}", self.matchers[pattern_pos]), self.debug);
                match &self.matchers[pattern_pos] {
                    Matcher::Skip | Matcher::ZeroOrOne(_) | Matcher::EndOfString  => (),
                    _ => {
                        has_non_wildcards = true;
                    }
                }
                pattern_pos += 1;
            }
            if has_non_wildcards {
                print_debug("Finished input but not pattern", self.debug);
                return (false, 0);
            }
            print_debug("No non-wildcards found", self.debug);
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
    Association(Vec<String>),
    Backref(u32)
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
                    Some(c) if c.is_digit(10) => {
                        let mut a = 1;
                        let mut s = String::new();
                        while pos + a < input.chars().count() && input.chars().nth(pos + a).unwrap().is_digit(10) {
                            s.push(input.chars().nth(pos + a).unwrap());
                            a += 1;
                        }
                        (Self::Backref(s.parse().unwrap()), a - 1)
                    }
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
                    match input.chars().nth(pos + 1) {
                        Some('^') => (Self::GroupNegative(&input[pos + 2..closing_bracket]), closing_bracket - pos),
                        Some(_) => (Self::GroupPositive(&input[pos + 1..closing_bracket]), closing_bracket - pos),
                        None => panic!("This cannot happen")
                    }
                }
                else {
                    panic!("Unclosed [");
                }
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

    fn test(&self, input: &str, pos: usize, backrefs_map: &HashMap<u32, String>, debug: bool) -> (bool, usize) {
        match self {
            Self::Literal(pc) => {
                match input.chars().nth(pos) {
                    Some(c) => {
                        let result = pc == &c;
                        print_debug(&format!("Match[Literal] {} {} with {}", result, pc, c), debug);
                        (result, 1)
                    },
                    None => (false, 0)
                }
            }
            Self::Digit => {
                match input.chars().nth(pos) {
                    Some(c) => {
                        let result = c.is_digit(10);
                        print_debug(&format!("Match[Digit] true {} \\d with {}", result, c), debug);
                        (result, 1)
                    },
                    None => (false, 0)
                }
            }
            Self::Word => {
                match input.chars().nth(pos) {
                    Some(c) => {
                        let result = c.is_alphabetic() || c.is_digit(10);
                        print_debug(&format!("Match[Word] true {} \\w with {}", result, c), debug);
                        (result, 1)
                    },
                    None => {
                        print_debug(&format!("Match[Word] false for \\w at position {}",  pos), debug);
                        (false, 0)
                    }
                }
            }
            Self::GroupPositive(group) => {
                match input.chars().nth(pos) {
                    Some(c) => {
                        let result = group.contains(c);
                        print_debug(&format!("Match[GroupPositive] true {} [{}] with {}", result, group, c), debug);
                        (result, 1)
                    },
                    None => {
                        print_debug(&format!("Match[GroupPositive] false for [{}] at position {}", group, pos), debug);
                        (false, 0)
                    }
                }
            }
            Self::GroupNegative(group) => {
                match input.chars().nth(pos) {
                    Some(c) => {
                        let result = !group.contains(c);
                        print_debug(&format!("Match[GroupNegative] true {} [^{}] with {}", result, group, c), debug);
                        (result, 1)
                    },
                    None => {
                        print_debug(&format!("Match[GroupNegative] false for [^{}] at position {}", group, pos), debug);
                        (false, 0)
                    }
                }
            }
            Self::EndOfString => {
                // cats cat$
                let result = pos >= input.chars().count();
                print_debug(&format!("Match[EndOfString] true {} $ at position {}", result, pos), debug);
                (result, 1)
            }
            Self::WildCard => {
                match input.chars().nth(pos) {
                    Some(c) => {
                        print_debug(&format!("Match[WildCard] true . with {}", c), debug);
                        (true, 1)
                    },
                    None => {
                        print_debug(&format!("Match[WildCard] false for . at position {}", pos), debug);
                        (false, 0)
                    }
                }
            }
            Self::OneOrMore(pc) => {
                let mut matched = 0;
                return match input.chars().nth(pos) {
                    Some(_) => {
                        let (result, advance) = pc.test(&input, pos, &backrefs_map, debug);
                        if result {
                            matched += advance;
                            while pos + matched < input.chars().count() {
                                let (result, advance) = pc.test(&input, pos + matched, &backrefs_map, debug);
                                if result {
                                    matched += advance;
                                }
                                else {
                                    break;
                                }
                            }
                        }
                        return if matched == 0 {
                            print_debug(&format!("Match[OneOrMore] false {:?}+", *pc),  debug);
                            (false, 1)
                        } else {
                            print_debug(&format!("Match[OneOrMore] true {:?}+ {} times", *pc, matched), debug);
                            (true, matched)
                        }
                    },
                    None => {
                        print_debug(&format!("Match[OneOrMore] false for {:?}+ at position {}", *pc, pos), debug);
                        (false, 0)
                    }
                }
            }
            Self::ZeroOrOne(pc) => {
                //doa?g - dog
                match input.chars().nth(pos) {
                    Some(c) => {
                        let (result, advance) = pc.test(&input, pos, &backrefs_map, debug);
                        if result {
                            print_debug(&format!("Match[ZeroOrOne] true {:?}? with {} ", *pc, c), debug);
                            (true, advance)
                        }
                        else {
                            print_debug(&format!("Match[ZeroOrOne] (Optional) false {:?}? with {} ", *pc, c), debug);
                            (true, 0)
                        }
                    },
                    None => {
                        print_debug(&format!("Match[ZeroOrOne] (Optional) false {:?}? at end of input ", *pc), debug);
                        (true, 0)
                    }
                }
            }
            Self::Association(parts) => {
                for sub_pattern in parts {
                    let sub_input = &input[pos..].to_string();
                    let mut sub_pattern_matcher = Pattern::new(sub_pattern, debug);
                    let (result, to_advance) = sub_pattern_matcher.test(&sub_input);
                    if result {
                        print_debug(&format!("Match[Association] true subpattern {} from association group with {} characters at position {}", sub_pattern, to_advance, pos), debug);
                        return (true, to_advance);
                    } else {
                        print_debug(&format!("Match[Association] false subpattern {} from association group at position {}", sub_pattern, pos), debug);
                    }
                }
                print_debug(&format!("Match[Association] false entire association group at position {}",  pos), debug);
                (false, 1)
            },
            Self::Backref(backref_pos) => {
                if let Some(s) = backrefs_map.get(backref_pos) {
                    let sub_input = &input[pos..].to_string();
                    let mut sub_pattern_matcher = Pattern::new(s, debug);
                    let (result, to_advance) = sub_pattern_matcher.test(&sub_input);
                    if result {
                        print_debug(&format!("Match[Backref] true backref {}", s), debug);
                        return (true, to_advance);
                    } else {
                        print_debug(&format!("Match[Backref] false backref {}", s), debug);
                    }
                }
                return (false, 1);
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
        debug!("{}", message);
    }
}