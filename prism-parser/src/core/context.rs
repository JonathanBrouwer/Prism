use crate::core::span::Span;
use crate::core::tokens::{Token, TokenType, Tokens};
use crate::parsable::parsed::Parsed;
use crate::parser::VarMap;
use std::hash::Hash;
use std::sync::Arc;

#[derive(Clone)]
pub struct PR {
    pub free: VarMap,
    pub rtrn: PV,
}

impl PR {
    pub fn with_rtrn(rtrn: PV) -> Self {
        Self {
            free: VarMap::default(),
            rtrn,
        }
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct ParserContext {
    pub recovery_disabled: bool,
    pub layout_disabled: bool,
}

impl Default for ParserContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ParserContext {
    pub fn new() -> Self {
        Self {
            recovery_disabled: false,
            layout_disabled: false,
        }
    }
}

#[derive(Clone)]
pub struct PV {
    pub parsed: Parsed,
    pub tokens: Arc<Tokens>,
}

impl PV {
    pub fn new_single(rtrn: Parsed, token_type: TokenType, span: Span) -> Self {
        Self {
            parsed: rtrn,
            tokens: Arc::new(Tokens::Single(Token { token_type, span })),
        }
    }

    pub fn new_multi(rtrn: Parsed, tokens: Vec<Arc<Tokens>>) -> Self {
        Self {
            parsed: rtrn,
            tokens: Arc::new(Tokens::Multi(tokens)),
        }
    }

    pub fn new_from(rtrn: Parsed, tokens: Arc<Tokens>) -> Self {
        Self {
            parsed: rtrn,
            tokens,
        }
    }
}
