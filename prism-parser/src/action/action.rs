use crate::core::adaptive::RuleId;
use crate::core::span::Span;
use crate::grammar::escaped_string::EscapedString;

pub trait ActionVisitor<'arn, 'grm>: Sized {
    fn visit_value(&mut self, span: Span) -> Self;
    
    fn visit_literal(&mut self, literal: EscapedString<'grm>) -> Self;
    
    fn visit_construct(&mut self, span: Span, name: &'grm str, args: &'arn [Self]) -> Self;

    fn visit_guid(&mut self, guid: usize) -> Self;

    fn visit_rule(&mut self, rule: RuleId) -> Self;
}
