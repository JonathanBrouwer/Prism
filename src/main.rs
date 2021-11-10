use std::cmp::Ordering;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use crate::ParseErrorFlag::{Recursive, NotAllInput};

fn main() {

}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
enum Input {
    A,B,C
}

impl Display for Input {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Input::A => write!(f, "A"),
            Input::B => write!(f, "B"),
            Input::C => write!(f, "C"),
        }

    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
enum PegRule {
    Terminal(Input),
    Sequence(Vec<usize>),
    Choice(Vec<usize>),
}

type PegRuleResult = Rc<PegRuleResultInner>;

#[derive(Debug, Eq, PartialEq)]
enum PegRuleResultInner {
    Terminal(Input),
    Sequence(Vec<PegRuleResult>),
    Choice(usize, PegRuleResult)
}

impl Display for PegRuleResultInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PegRuleResultInner::Terminal(inp) => write!(f, "{}", inp),
            PegRuleResultInner::Sequence(seq) => {
                write!(f, "[")?;
                for (i, res) in seq.iter().enumerate() {
                    if i == 0 { write!(f, "{}", res)?; }
                    else { write!(f, " {}", res)?; }
                }
                write!(f, "]")
            }
            PegRuleResultInner::Choice(i, res) => {
                write!(f, "<{} {}>", i, res)
            }
        }
    }
}

struct PegParser {
    rules: Vec<PegRule>,
    input: &'static [Input],
}

struct PegParserStateEntry {
    result: Result<ParseSuccess, ParseError>,
    used: bool
}
struct PegParserState {
    memtable: HashMap<(usize, usize), PegParserStateEntry>,
    changed_stack: Vec<(usize, usize)>
}

impl PegParser {
    pub fn new(rules: Vec<PegRule>, input: &'static [Input]) -> Self {
        Self::validate_peg_rules(&rules);
        Self{ rules, input }
    }

    fn validate_peg_rules(rules: &Vec<PegRule>) {
        let validate_range = |num: usize| if num >= rules.len() { panic!("Invalid rules.")};
        let validate_not_empty = |num: &Vec<usize>| if num.is_empty() { panic!("Invalid rules.")};
        for rule in rules {
            match rule {
                PegRule::Terminal(_) => {}
                PegRule::Sequence(ns) => ns.iter().for_each(|n| validate_range(*n)),
                PegRule::Choice(ns) => {
                    validate_not_empty(ns);
                    ns.iter().for_each(|n| validate_range(*n));
                },
            }
        }
    }

    pub fn parse_final(self) -> Result<PegRuleResult, ParseError> {
        let mut state = PegParserState { memtable: HashMap::new(), changed_stack: Vec::new() };
        let suc = self.parse(&mut state, 0, self.rules.len() - 1)?;
        if suc.rest < self.input.len() {
            return Err(ParseError {
                positives: vec![],
                flags: vec![NotAllInput],
                location: suc.rest })
        }
        Ok(suc.result)
    }

    pub fn parse(&self, state: &mut PegParserState, index: usize, rule: usize) -> Result<ParseSuccess, ParseError> {
        //Check memtable
        if let Some(entry) = state.memtable.get_mut(&(index, rule)) {
            entry.used = true;
            return entry.result.clone();
        }

        //Insert temp entry
        state.memtable.insert((index, rule), PegParserStateEntry { result: Err(ParseError {
            positives: vec![],
            flags: vec![Recursive],
            location: index }), used: false });

        //Create seed
        let stack_len_before = state.changed_stack.len();
        let mut res = self.parse_inner(state, index, rule);

        //Grow seed if needed
        let entry = state.memtable.get_mut(&(index, rule)).unwrap();
        if entry.used && res.is_ok() {
            loop {
                //Invariant: res is ok.

                //Store old rest
                let old_rest = if let Ok(ok) = &res {ok.rest} else {unreachable!()};

                //Remove old memory, insert current seed into state
                state.changed_stack.drain(stack_len_before..).for_each(|x| {state.memtable.remove(&x);});
                state.memtable.insert((index, rule), PegParserStateEntry { result: res.clone(), used: false });

                //Grow the seed
                let new_res = self.parse_inner(state, index, rule);

                //Check if it grew
                let new_rest = if let Ok(ok) = &new_res {ok.rest} else { break };
                if new_rest <= old_rest { break }
                res = new_res;
            }
            state.changed_stack.drain(stack_len_before..).for_each(|x| {state.memtable.remove(&x);});
        }

        //Store result
        let entry = state.memtable.get_mut(&(index, rule)).unwrap();
        state.changed_stack.push((index, rule));
        entry.result = res.clone();
        res
    }

    pub fn parse_inner(&self, state: &mut PegParserState, index: usize, rule: usize) -> Result<ParseSuccess, ParseError> {
        match &self.rules[rule] {
            &PegRule::Terminal(expect) => {
                if index < self.input.len() && self.input[index] == expect {
                    Ok(ParseSuccess {
                        result: Rc::new(PegRuleResultInner::Terminal(expect)),
                        best_error: None,
                        rest: index + 1
                    })
                } else {
                    Err(ParseError{
                        positives: vec![expect],
                        flags: vec![],
                        location: index
                    })
                }
            }
            PegRule::Sequence(subrules) => {
                let mut result = Vec::new();
                let mut best_error = None;
                let mut rest = index;
                for subrule in subrules {
                    match self.parse(state, rest, *subrule) {
                        Ok(succ) => {
                            result.push(succ.result);
                            best_error = parse_error_combine_opt2(best_error, succ.best_error);
                            rest = succ.rest;
                        }
                        Err(err) => {
                            return Err(parse_error_combine_opt1(err, best_error));
                        }
                    }
                }
                Ok(ParseSuccess {
                    result: Rc::new(PegRuleResultInner::Sequence(result)), best_error, rest
                })
            }
            PegRule::Choice(subrules) => {
                let mut best_error = None;
                for (i, subrule) in subrules.iter().enumerate() {
                    match self.parse(state, index, *subrule) {
                        Ok(succ) => {
                            best_error = parse_error_combine_opt2(best_error, succ.best_error);
                            return Ok(ParseSuccess {
                                result: Rc::new(PegRuleResultInner::Choice(i, succ.result)), best_error, rest: succ.rest
                            })
                        }
                        Err(err) => {
                            best_error = Some(parse_error_combine_opt1(err, best_error));
                        }
                    }
                }
                return Err(best_error.unwrap());
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct ParseSuccess {
    result: PegRuleResult,
    best_error: Option<ParseError>,
    rest: usize
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct ParseError {
    positives: Vec<Input>,
    flags: Vec<ParseErrorFlag>,
    location: usize,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum ParseErrorFlag {
    Recursive,
    NotAllInput
}

fn parse_error_combine_opt2(e1: Option<ParseError>, e2: Option<ParseError>) -> Option<ParseError> {
    match (e1, e2) {
        (Some(e1), Some(e2)) => Some(parse_error_combine(e1, e2)),
        (Some(e1), None) => Some(e1),
        (None, Some(e2)) => Some(e2),
        (None, None) => None,
    }
}

fn parse_error_combine_opt1(e1: ParseError, e2: Option<ParseError>) -> ParseError {
    match e2 {
        Some(e2) => parse_error_combine(e1, e2),
        None => e1
    }
}

fn parse_error_combine(mut e1: ParseError, mut e2: ParseError) -> ParseError {
    match e1.location.cmp(&e2.location) {
        Ordering::Less => e2,
        Ordering::Greater => e1,
        Ordering::Equal => {
            e1.positives.append(&mut e2.positives);
            e1.flags.append(&mut e2.flags);
            e1
        }
    }
}

enum InputLocation {
    Pos(usize),
    Span((usize, usize))
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use crate::{Input, ParseError, ParseSuccess, PegParser, PegRule, PegRuleResult};
    use crate::Input::{A, B, C};
    use crate::ParseErrorFlag::{Recursive, NotAllInput};
    use crate::PegRuleResultInner::{Choice, Sequence, Terminal};

    #[test]
    fn test_terminal1() {
        let rules = vec![
            PegRule::Terminal(A),
        ];
        assert_eq!(
            PegParser::new(rules.clone(),&[A]).parse_final().map(|ok| ok.to_string()),
            Ok(String::from("A")),
        );
        assert_eq!(
            PegParser::new(rules.clone(),&[B]).parse_final(),
            Err(ParseError { positives: vec![A], flags: vec![], location: 0 }),
        );
    }

    #[test]
    fn test_sequence1() {
        let rules = vec![
            PegRule::Terminal(A),
            PegRule::Sequence(vec![0, 0]),
        ];
        assert_eq!(
            PegParser::new(rules.clone(),&[A, A]).parse_final().map(|ok| ok.to_string()),
            Ok(String::from("[A A]"))
        );
        assert_eq!(
            PegParser::new(rules.clone(),&[B, A]).parse_final(),
            Err(ParseError { positives: vec![A], flags: vec![], location: 0 }),
        );
        assert_eq!(
            PegParser::new(rules.clone(),&[A, B]).parse_final(),
            Err(ParseError { positives: vec![A], flags: vec![], location: 1 }),
        );
    }

    #[test]
    fn test_choice1() {
        let rules = vec![
            PegRule::Terminal(A),
            PegRule::Terminal(B),
            PegRule::Choice(vec![0, 1]),
        ];
        assert_eq!(
            PegParser::new(rules.clone(),&[A]).parse_final().map(|ok| ok.to_string()),
            Ok(String::from("<0 A>")),
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[B]).parse_final().map(|ok| ok.to_string()),
            Ok(String::from("<1 B>")),
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[C]).parse_final(),
            Err(ParseError { positives: vec![B, A], flags: vec![], location: 0 }),
        );
    }

    #[test]
    fn test_rightrec() {
        let rules = vec![
            PegRule::Terminal(A),
            PegRule::Terminal(B),
            PegRule::Sequence(vec![0, 3]),
            PegRule::Choice(vec![2, 1]),
        ];
        assert_eq!(
            PegParser::new(rules.clone(), &[B]).parse_final().map(|ok| ok.to_string()),
            Ok(String::from("<1 B>"))
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[A, B]).parse_final().map(|ok| ok.to_string()),
            Ok(String::from("<0 [A <1 B>]>"))
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[A, A, B]).parse_final().map(|ok| ok.to_string()),
            Ok(String::from("<0 [A <0 [A <1 B>]>]>"))
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[B, A]).parse_final(),
            Err(ParseError { positives: vec![], flags: vec![NotAllInput], location: 1 })
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[A]).parse_final(),
            Err(ParseError { positives: vec![B, A], flags: vec![], location: 1 })
        );
    }

    #[test]
    fn test_leftrec() {
        let rules = vec![
            PegRule::Terminal(A),
            PegRule::Terminal(B),
            PegRule::Sequence(vec![3, 0]),
            PegRule::Choice(vec![2, 1]),
        ];
        assert_eq!(
            PegParser::new(rules.clone(), &[B]).parse_final().map(|ok| ok.to_string()),
            Ok(String::from("<1 B>"))
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[B, A]).parse_final().map(|ok| ok.to_string()),
            Ok(String::from("<0 [<1 B> A]>"))
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[A, B]).parse_final(),
            Err(ParseError { positives: vec![B], flags: vec![Recursive], location: 0 })
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[A]).parse_final(),
            Err(ParseError { positives: vec![B], flags: vec![Recursive], location: 0 })
        );
    }

    #[test]
    fn test_leftrec_unavoidable() {
        let rules = vec![
            PegRule::Sequence(vec![0])
        ];
        assert_eq!(
            PegParser::new(rules.clone(), &[A]).parse_final(),
            Err(ParseError { positives: vec![], flags: vec![Recursive], location: 0 })
        );
    }

    #[test]
    fn test_notall() {
        let rules = vec![
            PegRule::Sequence(vec![])
        ];
        assert_eq!(
            PegParser::new(rules.clone(), &[A]).parse_final(),
            Err(ParseError { positives: vec![], flags: vec![NotAllInput], location: 0 })
        );
    }
}