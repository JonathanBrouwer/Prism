use crate::lang::{PartialExpr, TcEnv, UnionIndex};
use crate::parser::parse_env::ParsedEnv;
use prism_parser::core::cache::Allocs;
use prism_parser::core::input::Input;
use prism_parser::core::pos::Pos;
use prism_parser::core::span::Span;
use prism_parser::parsable::env_capture::EnvCapture;
use prism_parser::parsable::parsed::Parsed;
use prism_parser::parsable::{Parsable2, ParseResult};

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for UnionIndex {}
impl<'arn, 'grm: 'arn> Parsable2<'arn, 'grm, TcEnv<'grm>> for UnionIndex {
    fn from_construct(
        span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        allocs: Allocs<'arn>,
        _src: &'grm str,
        tc_env: &mut TcEnv<'grm>,
    ) -> Result<Self, String> {
        let expr: PartialExpr<'grm> = match constructor {
            "Type" => {
                assert_eq!(args.len(), 0);

                PartialExpr::Type
            }
            "Name" => {
                assert_eq!(args.len(), 1);
                let name = reduce_expr(args[0], tc_env, allocs)
                    .into_value::<Input>()
                    .as_str(_src);
                if name == "_" {
                    PartialExpr::Free
                } else {
                    PartialExpr::Name(name)
                }
            }
            "Let" => {
                assert_eq!(args.len(), 3);
                let name = reduce_expr(args[0], tc_env, allocs)
                    .into_value::<Input<'grm>>()
                    .as_str(_src);
                let v = *reduce_expr(args[1], tc_env, allocs).into_value::<UnionIndex>();
                let b = *reduce_expr(args[2], tc_env, allocs).into_value::<UnionIndex>();
                PartialExpr::Let(name, v, b)
            }
            "FnType" => {
                assert_eq!(args.len(), 3);
                let name = reduce_expr(args[0], tc_env, allocs)
                    .into_value::<Input<'grm>>()
                    .as_str(_src);
                let v = *reduce_expr(args[1], tc_env, allocs).into_value::<UnionIndex>();
                let b = *reduce_expr(args[2], tc_env, allocs).into_value::<UnionIndex>();
                PartialExpr::FnType(name, v, b)
            }
            "FnConstruct" => {
                assert_eq!(args.len(), 2);
                let name = reduce_expr(args[0], tc_env, allocs)
                    .into_value::<Input<'grm>>()
                    .as_str(_src);
                let b = *reduce_expr(args[1], tc_env, allocs).into_value::<UnionIndex>();
                PartialExpr::FnConstruct(name, b)
            }
            "FnDestruct" => {
                assert_eq!(args.len(), 2);
                let f = *reduce_expr(args[0], tc_env, allocs).into_value::<UnionIndex>();
                let v = *reduce_expr(args[1], tc_env, allocs).into_value::<UnionIndex>();
                PartialExpr::FnDestruct(f, v)
            }
            "TypeAssert" => {
                assert_eq!(args.len(), 2);

                let e = *reduce_expr(args[0], tc_env, allocs).into_value::<UnionIndex>();
                let typ = *reduce_expr(args[1], tc_env, allocs).into_value::<UnionIndex>();
                PartialExpr::TypeAssert(e, typ)
            }
            _ => unreachable!(),
        };

        Ok(tc_env.store_from_source(expr, span))
    }
}

pub fn reduce_expr<'arn, 'grm: 'arn>(
    parsed: Parsed<'arn, 'grm>,
    tc_env: &mut TcEnv,
    allocs: Allocs<'arn>,
) -> Parsed<'arn, 'grm> {
    if let Some(v) = parsed.try_into_value::<EnvCapture>() {
        let value = v.value.into_value::<ScopeEnter<'arn, 'grm>>();
        let from_env = value.1;
        let to_env = v.env.get("env").unwrap().into_value::<ParsedEnv>();

        let shift = to_env.find_shift_to(from_env);
        let inner = *reduce_expr(value.0, tc_env, allocs).into_value::<UnionIndex>();

        let expr = tc_env.store_from_source(
            PartialExpr::Shift(inner, shift),
            Span::new(Pos::invalid(), Pos::invalid()),
        );
        Parsed::from_value(allocs.alloc(expr))
    } else {
        parsed
    }
}

#[derive(Copy, Clone)]
pub struct ScopeEnter<'arn, 'grm>(Parsed<'arn, 'grm>, &'arn ParsedEnv<'arn>);
impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for ScopeEnter<'arn, 'grm> {}
impl<'arn, 'grm: 'arn> Parsable2<'arn, 'grm, TcEnv<'grm>> for ScopeEnter<'arn, 'grm> {
    fn from_construct(
        span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        allocs: Allocs<'arn>,
        src: &'grm str,
        tc_env: &mut TcEnv,
    ) -> Result<Self, String> {
        assert_eq!(constructor, "Enter");
        Ok(ScopeEnter(args[0], args[1].into_value::<ParsedEnv<'arn>>()))
    }
}
