use std::cmp::Ordering;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

fn main() {

}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
enum Input {
    A,B,C
}

#[derive(Eq, PartialEq, Debug, Clone)]
enum PegRule {
    Terminal(Input),
    Sequence(Vec<usize>),
    Choice(Vec<usize>),
}

#[derive(Debug, Eq, PartialEq, Clone)]
enum PegRuleResult {
    Terminal,
    Sequence(Vec<PegRuleResult>),
    Choice(usize, Box<PegRuleResult>)
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
                customs: vec![String::from("Did not parse full input.")],
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
            customs: vec![String::from("Hit left recursion.")],
            location: index }), used: false });

        //Create seed
        let res = self.parse_inner(state, index, rule);

        //Grow seed if needed
        let entry = state.memtable.get_mut(&(index, rule)).unwrap();
        if entry.used {
            panic!("Left recursion.");
        }

        //Store result
        entry.result = res.clone();
        res
    }

    pub fn parse_inner(&self, state: &mut PegParserState, index: usize, rule: usize) -> Result<ParseSuccess, ParseError> {
        match &self.rules[rule] {
            &PegRule::Terminal(expect) => {
                if index < self.input.len() && self.input[index] == expect {
                    Ok(ParseSuccess {
                        result: PegRuleResult::Terminal,
                        best_error: None,
                        rest: index + 1
                    })
                } else {
                    Err(ParseError{
                        positives: vec![expect],
                        customs: vec![],
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
                    result: PegRuleResult::Sequence(result), best_error, rest
                })
            }
            PegRule::Choice(subrules) => {
                let mut best_error = None;
                for (i, subrule) in subrules.iter().enumerate() {
                    match self.parse(state, index, *subrule) {
                        Ok(succ) => {
                            best_error = parse_error_combine_opt2(best_error, succ.best_error);
                            return Ok(ParseSuccess {
                                result: PegRuleResult::Choice(i, Box::new(succ.result)), best_error, rest: succ.rest
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
    customs: Vec<String>,
    location: usize,
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
            e1.customs.append(&mut e2.customs);
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
    use crate::{Input, ParseError, ParseSuccess, PegParser, PegRule, PegRuleResult};
    use crate::Input::{A, B, C};
    use crate::PegRuleResult::{Choice, Sequence, Terminal};

    #[test]
    fn test_terminal1() {
        let rules = vec![
            PegRule::Terminal(A),
        ];
        assert_eq!(
            PegParser::new(rules.clone(),&[A]).parse_final(),
            Ok(Terminal),
        );
        assert_eq!(
            PegParser::new(rules.clone(),&[B]).parse_final(),
            Err(ParseError { positives: vec![A], customs: vec![], location: 0 }),
        );
    }

    #[test]
    fn test_sequence1() {
        let rules = vec![
            PegRule::Terminal(A),
            PegRule::Sequence(vec![0, 0]),
        ];
        assert_eq!(
            PegParser::new(rules.clone(),&[A, A]).parse_final(),
            Ok(Sequence(vec![Terminal, Terminal])),
        );
        assert_eq!(
            PegParser::new(rules.clone(),&[B, A]).parse_final(),
            Err(ParseError { positives: vec![A], customs: vec![], location: 0 }),
        );
        assert_eq!(
            PegParser::new(rules.clone(),&[A, B]).parse_final(),
            Err(ParseError { positives: vec![A], customs: vec![], location: 1 }),
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
            PegParser::new(rules.clone(),&[A]).parse_final(),
            Ok(Choice(0, Box::new(Terminal))),
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[B]).parse_final(),
            Ok(Choice(1, Box::new(Terminal))),
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[C]).parse_final(),
            Err(ParseError { positives: vec![B, A], customs: vec![], location: 0 }),
        );
    }

    #[test]
    fn test_rightrec() {
        let rules = vec![
            PegRule::Terminal(A),
            PegRule::Terminal(B),
            PegRule::Sequence(vec![0, 4]),
            PegRule::Sequence(vec![1]),
            PegRule::Choice(vec![2, 3]),
        ];
        assert_eq!(
            PegParser::new(rules.clone(), &[B]).parse_final(),
            Ok(Choice(1, Box::new(Sequence(vec![Terminal])))),
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[A, B]).parse_final(),
            Ok(Choice(0, Box::new(Sequence(vec![Terminal, Choice(1, Box::new(Sequence(vec![Terminal])))]))))
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[B, A]).parse_final(),
            Err(ParseError { positives: vec![], customs: vec![String::from("Did not parse full input.")], location: 1 })
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[A]).parse_final(),
            Err(ParseError { positives: vec![B, A], customs: vec![], location: 1 })
        );
    }

    #[test]
    fn test_leftrec() {
        let rules = vec![
            PegRule::Terminal(A),
            PegRule::Terminal(B),
            PegRule::Sequence(vec![4, 0]),
            PegRule::Sequence(vec![1]),
            PegRule::Choice(vec![2, 3]),
        ];
        assert_eq!(
            PegParser::new(rules.clone(), &[B]).parse_final(),
            Ok(Choice(1, Box::new(Sequence(vec![Terminal])))),
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[B, A]).parse_final(),
            Ok(Choice(0, Box::new(Sequence(vec![Terminal, Choice(1, Box::new(Sequence(vec![Terminal])))]))))
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[A, B]).parse_final(),
            Err(ParseError { positives: vec![], customs: vec![], location: 1 })
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[A]).parse_final(),
            Err(ParseError { positives: vec![B, A], customs: vec![], location: 1 })
        );
    }
}