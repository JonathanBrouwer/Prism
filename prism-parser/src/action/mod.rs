use std::iter;
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

    fn cache(&self) -> *const ();
    fn visit_cache(&mut self, value: *const ());
}

#[derive(Copy, Clone)]
pub struct IgnoreVisitor;
impl<'arn, 'grm> ActionVisitor<'arn, 'grm> for IgnoreVisitor {
    fn visit_input_str(&mut self, s: &'arn str, span: Span) {
    }

    fn visit_literal(&mut self, lit: EscapedString<'grm>) {
    }

    fn visit_construct<'a>(&'a mut self, name: &'grm str, arity_hint: usize, allocs: Allocs<'arn>) -> Vec<&'a mut (dyn ActionVisitor<'arn, 'grm> + 'a)> {
        iter::from_fn(|| Some(allocs.alloc(IgnoreVisitor) as &mut dyn ActionVisitor)).take(arity_hint).collect()
    }

    fn visit_guid(&mut self, guid: usize) {
    }

    fn cache(&self) -> *const () {
        &() as *const ()
    }

    fn visit_cache(&mut self, value: *const ()) {
    }
}