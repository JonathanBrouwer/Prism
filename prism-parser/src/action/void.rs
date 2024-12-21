use std::any::type_name;
use crate::action::parsable::{Parsable, Parsed};
use crate::core::adaptive::RuleId;
use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::grammar::escaped_string::EscapedString;

pub struct Void;

impl<'arn, 'grm: 'arn> Parsable<'arn, 'grm> for Void {
    fn from_span(_span: Span, _text: &'arn str, _allocs: Allocs<'arn>) -> Self {
        Self
    }

    fn from_literal(_literal: EscapedString<'grm>, _allocs: Allocs<'arn>) -> Self {
        Self
    }

    fn from_guid(_guid: usize, _allocs: Allocs<'arn>) -> Self {
        Self
    }

    fn from_rule(_rule: RuleId, _allocs: Allocs<'arn>) -> Self {
        Self
    }

    fn from_construct(_span: Span, constructor: &'grm str, _args: &[Parsed<'arn, 'grm>], _allocs: Allocs<'arn>) -> Self {
        Self
    }
}