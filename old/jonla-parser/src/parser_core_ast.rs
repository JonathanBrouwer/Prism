use crate::character_class::CharacterClass;
use crate::span::Span;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum CoreExpression<'src> {
    Name(&'src str),
    Sequence(Vec<CoreExpression<'src>>),
    Repeat {
        subexpr: Box<CoreExpression<'src>>,
        min: u64,
        max: Option<u64>,
    },
    CharacterClass(CharacterClass),
    Choice(Vec<CoreExpression<'src>>),
}

#[derive(Debug, Clone)]
pub struct CoreAst<'src> {
    pub sorts: HashMap<&'src str, CoreExpression<'src>>,
    pub starting_sort: &'src str,
}

#[derive(Debug, Clone)]
pub enum ParsePairRaw<'src> {
    Name(Span<'src>, Box<ParsePairRaw<'src>>),
    List(Span<'src>, Vec<ParsePairRaw<'src>>),
    Choice(Span<'src>, usize, Box<ParsePairRaw<'src>>),
    Empty(Span<'src>),
    Error(Span<'src>),
}

impl<'src> ParsePairRaw<'src> {
    /// What span does this parse pair occupy?
    pub fn span(&self) -> Span<'src> {
        match self {
            ParsePairRaw::Name(span, _) => span,
            ParsePairRaw::List(span, _) => span,
            ParsePairRaw::Choice(span, _, _) => span,
            ParsePairRaw::Empty(span) => span,
            ParsePairRaw::Error(span) => span,
        }
        .clone()
    }
}
