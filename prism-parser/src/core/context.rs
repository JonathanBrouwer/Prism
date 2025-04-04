use crate::core::span::Span;
use crate::parsable::parsed::Parsed;
use crate::parser::VarMap;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::slice;
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

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub struct ParserContext {
    pub(crate) recovery_disabled: bool,
    pub(crate) layout_disabled: bool,
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

#[derive(Clone, Debug)]
pub enum Tokens {
    Single(Token),
    Multi(Vec<Arc<Tokens>>),
}

#[derive(Clone, Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum TokenType {
    CharClass,
    Literal,
    Slice,
}

impl Tokens {
    pub fn to_vec(&self) -> Vec<Token> {
        fn insert_tokens(tokens: &Tokens, v: &mut Vec<Token>) {
            match tokens {
                Tokens::Single(token) => v.push(token.clone()),
                Tokens::Multi(tokens) => {
                    for token in tokens {
                        insert_tokens(token, v);
                    }
                }
            }
        }

        let mut tokens = vec![];
        insert_tokens(self, &mut tokens);
        tokens
    }
}

#[derive(Clone)]
pub struct PV {
    pub rtrn: Parsed,
    pub tokens: Arc<Tokens>,
}

impl PV {
    pub fn new_single(rtrn: Parsed, token_type: TokenType, span: Span) -> Self {
        Self {
            rtrn,
            tokens: Arc::new(Tokens::Single(Token { token_type, span })),
        }
    }

    pub fn new_multi(rtrn: Parsed, tokens: Vec<Arc<Tokens>>) -> Self {
        Self {
            rtrn,
            tokens: Arc::new(Tokens::Multi(tokens)),
        }
    }

    pub fn new_from(rtrn: Parsed, tokens: Arc<Tokens>) -> Self {
        Self { rtrn, tokens }
    }
}
