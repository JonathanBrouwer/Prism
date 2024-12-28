use crate::lang::env::Env;
use crate::lang::UnionIndex;
use prism_parser::core::cache::Allocs;
use prism_parser::core::span::Span;
use prism_parser::parsable::parsed::Parsed;
use prism_parser::parsable::Parsable;

#[derive(Copy, Clone)]
pub struct ParsedEnv<'arn>(Option<&'arn ParsedEnvNode<'arn>>);

impl<'arn> ParsedEnv<'arn> {
    pub fn new_empty() -> Self {
        Self(None)
    }
}

#[derive(Copy, Clone)]
struct ParsedEnvNode<'arn> {
    name: &'arn str,
    next: Option<&'arn ParsedEnvNode<'arn>>,
    value: ParsedEnvNodeValue<'arn>,
}

#[derive(Copy, Clone)]
enum ParsedEnvNodeValue<'arn> {
    Substitute {
        expr: UnionIndex,
        expr_env: ParsedEnv<'arn>,
    },
    Type,
}

impl<'arn, 'grm: 'arn> Parsable<'arn, 'grm> for ParsedEnv<'arn> {
    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        allocs: Allocs<'arn>,
        _src: &'grm str,
    ) -> Self {
        match constructor {
            "Nil" => ParsedEnv::new_empty(),
            "ConsSubstitute" => {
                assert_eq!(args.len(), 4);

                todo!()
            }
            "ConsType" => {
                assert_eq!(args.len(), 3);

                todo!()
            }
            _ => unreachable!(),
        }
    }
}
