use std::collections::HashMap;
use crate::grammar::{AnnotatedRuleExpr, Block, GrammarFile};

pub struct GrammarState<'grm> {
    pub rules: HashMap<&'grm str, Vec<BlockState<'grm>>>
}

impl<'grm> GrammarState<'grm> {
    pub fn new(grammar: &'grm GrammarFile) -> Self {
        Self {
            rules: grammar.rules.iter().map(|r| {
                let blocks: Vec<BlockState<'grm>> = r.blocks.iter().map(|b| BlockState::new(b)).collect();
                (&r.name[..], blocks)
            }).collect()
        }
    }
}

pub struct BlockState<'grm> {
    pub name: &'grm str,
    pub constructors: Vec<&'grm AnnotatedRuleExpr>
}

impl<'grm> BlockState<'grm> {
    pub fn new(block: &'grm Block) -> Self {
        Self {
            name: &block.0,
            constructors: block.1.iter().collect()
        }
    }
}