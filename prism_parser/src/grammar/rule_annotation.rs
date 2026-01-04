use crate::core::tokens::TokenType;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;
use prism_input::input::Input;
use prism_input::input_table::InputTable;
use prism_input::span::Span;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum RuleAnnotation {
    Token(TokenType),
}

impl<Db> Parsable<Db> for RuleAnnotation {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: &str,
        args: &[Parsed],
        _env: &mut Db,
        input: &InputTable,
    ) -> Self {
        match constructor {
            "Token" => {
                RuleAnnotation::Token(args[0].value_ref::<Input>().as_str(input).parse().unwrap())
            }
            _ => unreachable!(),
        }
    }
}
