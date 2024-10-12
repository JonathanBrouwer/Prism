use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::grammar::escaped_string::EscapedString;

pub mod action_result;
pub mod apply_action;

pub trait ActionVisitor<'arn, 'grm> {
    fn visit_input_str(&mut self, s: &'arn str, span: Span);
    fn visit_literal(&mut self, lit: EscapedString<'grm>);
    fn visit_construct<'a>(&'a mut self, name: &'grm str, arity_hint: usize, allocs: Allocs<'arn>) -> Vec<&'a mut (dyn ActionVisitor<'arn, 'grm> + 'a)>;
    fn visit_guid(&mut self, guid: usize);
}

pub struct IgnoreVisitor;
impl<'arn, 'grm> ActionVisitor<'arn, 'grm> for IgnoreVisitor {
    fn visit_input_str(&mut self, s: &'arn str, span: Span) {
    }

    fn visit_literal(&mut self, lit: EscapedString<'grm>) {
    }

    fn visit_construct<'a>(&'a mut self, name: &'grm str, arity_hint: usize, allocs: Allocs<'arn>) -> Vec<&'a mut (dyn ActionVisitor<'arn, 'grm> + 'a)> {
        vec![]
    }

    fn visit_guid(&mut self, guid: usize) {
    }
}