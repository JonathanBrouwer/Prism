use std::collections::HashMap;
use std::rc::Rc;
use crate::{Input, NotAllInput, ParseError, ParseSuccess, PegRule, PegRuleResult, PegRuleResultInner, Recursive};

pub struct PegParser<I: Input> {
    rules: Vec<PegRule<I::InputElement>>,
    input: I,
}

struct PegParserStateEntry<I: Input> {
    result: Result<ParseSuccess<I::InputElement>, ParseError<I::InputElement>>,
    used: bool,
}

struct PegParserState<I: Input> {
    memtable: HashMap<(usize, usize), PegParserStateEntry<I>>,
    changed_stack: Vec<(usize, usize)>,
}

impl<I: Input> PegParser<I> {
    pub fn new(rules: Vec<PegRule<I::InputElement>>, input: I) -> Self {
        Self::validate_peg_rules(&rules);
        Self { rules, input }
    }

    fn validate_peg_rules(rules: &Vec<PegRule<I::InputElement>>) {
        let validate_range = |num: usize| if num >= rules.len() { panic!("Invalid rules.") };
        let validate_not_empty = |num: &Vec<usize>| if num.is_empty() { panic!("Invalid rules.") };
        for rule in rules {
            match rule {
                PegRule::Terminal(_) => {}
                PegRule::Sequence(ns) => ns.iter().for_each(|n| validate_range(*n)),
                PegRule::Choice(ns) => {
                    validate_not_empty(ns);
                    ns.iter().for_each(|n| validate_range(*n));
                }
            }
        }
    }

    pub fn parse_final(self) -> Result<PegRuleResult<I::InputElement>, ParseError<I::InputElement>> {
        let mut state = PegParserState { memtable: HashMap::new(), changed_stack: Vec::new() };
        let suc = self.parse(&mut state, 0, self.rules.len() - 1)?;
        if self.input.next(suc.rest).is_some() {
            return Err(ParseError {
                positives: vec![],
                flags: vec![NotAllInput],
                location: suc.rest,
            });
        }
        Ok(suc.result)
    }

    fn parse(&self, state: &mut PegParserState<I>, index: usize, rule: usize) -> Result<ParseSuccess<I::InputElement>, ParseError<I::InputElement>> {
        //Check memtable
        if let Some(entry) = state.memtable.get_mut(&(index, rule)) {
            entry.used = true;
            return entry.result.clone();
        }

        //Insert temp entry
        state.memtable.insert((index, rule), PegParserStateEntry {
            result: Err(ParseError {
                positives: vec![],
                flags: vec![Recursive],
                location: index,
            }),
            used: false,
        });

        //Create seed
        let stack_len_before = state.changed_stack.len();
        let mut res = self.parse_inner(state, index, rule);

        //Grow seed if needed
        let entry = state.memtable.get_mut(&(index, rule)).unwrap();
        if entry.used && res.is_ok() {
            loop {
                //Invariant: res is ok.

                //Store old rest
                let old_rest = if let Ok(ok) = &res { ok.rest } else { unreachable!() };

                //Remove old memory, insert current seed into state
                state.changed_stack.drain(stack_len_before..).for_each(|x| { state.memtable.remove(&x); });
                state.memtable.insert((index, rule), PegParserStateEntry { result: res.clone(), used: false });

                //Grow the seed
                let new_res = self.parse_inner(state, index, rule);

                //Check if it grew
                let new_rest = if let Ok(ok) = &new_res { ok.rest } else { break; };
                if new_rest <= old_rest { break; }
                res = new_res;
            }
            state.changed_stack.drain(stack_len_before..).for_each(|x| { state.memtable.remove(&x); });
        }

        //Store result
        let entry = state.memtable.get_mut(&(index, rule)).unwrap();
        state.changed_stack.push((index, rule));
        entry.result = res.clone();
        res
    }

    fn parse_inner(&self, state: &mut PegParserState<I>, index: usize, rule: usize) -> Result<ParseSuccess<I::InputElement>, ParseError<I::InputElement>> {
        match &self.rules[rule] {
            &PegRule::Terminal(expect) => {
                match self.input.next(index) {
                    Some((elem, next_index)) if elem == expect => {
                        Ok(ParseSuccess {
                            result: Rc::new(PegRuleResultInner::Terminal(expect)),
                            best_error: None,
                            rest: next_index,
                        })
                    }
                    _ => {
                        Err(ParseError {
                            positives: vec![expect],
                            flags: vec![],
                            location: index,
                        })
                    }
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
                            best_error = ParseError::parse_error_combine_opt2(best_error, succ.best_error);
                            rest = succ.rest;
                        }
                        Err(err) => {
                            return Err(ParseError::parse_error_combine_opt1(err, best_error));
                        }
                    }
                }
                Ok(ParseSuccess {
                    result: Rc::new(PegRuleResultInner::Sequence(result)),
                    best_error,
                    rest,
                })
            }
            PegRule::Choice(subrules) => {
                let mut best_error = None;
                for (i, subrule) in subrules.iter().enumerate() {
                    match self.parse(state, index, *subrule) {
                        Ok(succ) => {
                            best_error = ParseError::parse_error_combine_opt2(best_error, succ.best_error);
                            return Ok(ParseSuccess {
                                result: Rc::new(PegRuleResultInner::Choice(i, succ.result)),
                                best_error,
                                rest: succ.rest,
                            });
                        }
                        Err(err) => {
                            best_error = Some(ParseError::parse_error_combine_opt1(err, best_error));
                        }
                    }
                }
                return Err(best_error.unwrap());
            }
        }
    }
}
