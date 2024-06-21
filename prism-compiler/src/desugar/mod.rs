use prism_parser::core::span::Span;

pub mod from_action_result;

#[derive(Default)]
pub struct ParseEnv {
    values: Vec<SourceExpr>,
    value_spans: Vec<Span>,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct ParseIndex(usize);

impl ParseIndex {
    pub fn index(self) -> usize {
        self.0
    }
}

impl ParseEnv {
    pub fn store(&mut self, e: SourceExpr, span: Span) -> ParseIndex {
        self.values.push(e);
        self.value_spans.push(span);
        ParseIndex(self.values.len() - 1)
    }

    pub fn values(&self) -> &[SourceExpr] {
        &self.values
    }

    pub fn value_spans(&self) -> &[Span] {
        &self.value_spans
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum SourceExpr {
    Type,
    Let(String, ParseIndex, ParseIndex),
    Variable(String),
    FnType(String, ParseIndex, ParseIndex),
    FnConstruct(String, ParseIndex, ParseIndex),
    FnDestruct(ParseIndex, ParseIndex),
    ScopeStart(ParseIndex, Guid),
    ScopeJump(ParseIndex, Guid),
}

#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Guid(usize);