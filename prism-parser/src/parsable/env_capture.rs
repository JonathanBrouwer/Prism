use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable2, ParseResult};
use crate::parser::var_map::VarMap;

#[derive(Clone, Copy)]
pub struct EnvCapture<'arn, 'grm> {
    pub env: VarMap<'arn, 'grm>,
    pub value: Parsed<'arn, 'grm>,
}

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for EnvCapture<'arn, 'grm> {}
