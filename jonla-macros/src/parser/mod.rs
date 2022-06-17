use crate::grammar::{CharClass, RuleBody};
use crate::parser::parser_core::ParserState;

mod parser_core;
mod parser_result;

impl<'src> ParserState<'src> {
    pub fn parse_expr<'grm>(&mut self, pos: usize, expr: &RuleBody<'grm>) {
        match expr {
            RuleBody::Rule(_) => {}
            RuleBody::CharClass(cc) => {

            }
            RuleBody::Literal(_) => {}
            RuleBody::Repeat { .. } => {}
            RuleBody::Sequence(_) => {
                let mut state = ParserState::new("Hey");
                state.parse_sequence(0, Box::new(|s, e| s.parse_charclass(e, &CharClass {
                    ranges: vec![]
                })));

            }
            RuleBody::Choice(_) => {}
            RuleBody::NameBind(_, _) => {}
            RuleBody::Action(_, _) => {}
        }
    }
}

