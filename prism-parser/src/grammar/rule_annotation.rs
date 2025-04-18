use crate::core::input::Input;
use crate::core::span::Span;
use crate::core::tokens::TokenType;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum RuleAnnotation {
    Token(TokenType),
}

impl<Db> Parsable<Db> for RuleAnnotation {
    type EvalCtx = ();

    fn from_construct(_span: Span, constructor: &Input, args: &[Parsed], _env: &mut Db) -> Self {
        match constructor.as_str() {
            "Token" => {
                RuleAnnotation::Token(args[0].value_ref::<Input>().as_str().parse().unwrap())
            }
            _ => unreachable!(),
        }
    }

    fn error_fallback(_env: &mut Db, _span: Span) -> Self {
        RuleAnnotation::Token(TokenType::Slice)
    }
}
