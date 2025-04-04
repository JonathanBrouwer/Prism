use crate::parsable::parsed::Parsed;
use crate::parser::VarMap;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

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

#[derive(Clone)]
pub struct Tokens {}

#[derive(Clone)]
pub struct PV {
    pub rtrn: Parsed,
    tokens: Tokens,
}

impl PV {
    pub fn new(rtrn: Parsed) -> Self {
        Self {
            rtrn,
            tokens: Tokens {},
        }
    }
}
