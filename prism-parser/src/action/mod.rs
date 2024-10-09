use erased::Erased;
use crate::core::span::Span;
use crate::grammar::escaped_string::EscapedString;

pub mod action_result;
pub mod apply_action;

pub struct ActionVisitor<'arn, 'grm> {
    pub visit_input_str: fn(s: &'arn str) -> Erased<'arn>,
    pub visit_literal: fn(EscapedString<'grm>) -> Erased<'arn>,
    pub visit_construct: fn(name: &str)
}