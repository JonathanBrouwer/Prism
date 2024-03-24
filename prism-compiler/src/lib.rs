use crate::coc::{TcEnv};
use lazy_static::lazy_static;
use prism_parser::error::error_printer::print_set_error;
use prism_parser::grammar::GrammarFile;
use prism_parser::parse_grammar;
use prism_parser::parser::parser_instance::{run_parser_rule};
use prism_parser::rule_action::RuleAction;

pub mod coc;
pub mod union_find;

lazy_static! {
    pub static ref GRAMMAR: GrammarFile<'static, RuleAction<'static, 'static>> = {
        let grammar = include_str!("../resources/grammar");
        match parse_grammar(grammar) {
            Ok(ok) => ok,
            Err(es) => {
                for e in es {
                    print_set_error(e, grammar, false);
                }
                panic!()
            }
        }
    };
}

pub fn parse_prism(program: &str) -> Option<TcEnv> {
    let expr: Result<_, _> = run_parser_rule(&GRAMMAR, "block", program, |r| {
        TcEnv::from_action_result(r, program)
    });
    match expr {
        Ok(o) => Some(o),
        Err(es) => {
            for e in es {
                print_set_error(e, program, false)
            }
            None
        }
    }
}
