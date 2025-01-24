use crate::lang::{PrismEnv, PrismExpr, UnionIndex};
use prism_parser::core::cache::Allocs;
use prism_parser::core::input::Input;
use prism_parser::core::span::Span;
use prism_parser::parsable::env_capture::EnvCapture;
use prism_parser::parsable::guid::Guid;
use prism_parser::parsable::parsed::Parsed;
use prism_parser::parsable::{Parsable, ParseResult};

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for UnionIndex {}
impl<'arn, 'grm: 'arn> Parsable<'arn, 'grm, PrismEnv<'arn, 'grm>> for UnionIndex {
    fn from_construct(
        span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        allocs: Allocs<'arn>,
        _src: &'grm str,
        tc_env: &mut PrismEnv<'arn, 'grm>,
    ) -> Self {
        let expr: PrismExpr<'arn, 'grm> = match constructor {
            "Type" => {
                assert_eq!(args.len(), 0);

                PrismExpr::Type
            }
            "Name" => {
                assert_eq!(args.len(), 1);
                let name = reduce_expr(args[0], tc_env, allocs)
                    .into_value::<Input>()
                    .as_str(_src);
                if name == "_" {
                    PrismExpr::Free
                } else {
                    PrismExpr::Name(name)
                }
            }
            "Let" => {
                assert_eq!(args.len(), 3);
                let name = reduce_expr(args[0], tc_env, allocs)
                    .into_value::<Input<'grm>>()
                    .as_str(_src);
                let v = *reduce_expr(args[1], tc_env, allocs).into_value::<UnionIndex>();
                let b = *reduce_expr(args[2], tc_env, allocs).into_value::<UnionIndex>();
                PrismExpr::Let(name, v, b)
            }
            "FnType" => {
                assert_eq!(args.len(), 3);
                let name = reduce_expr(args[0], tc_env, allocs)
                    .into_value::<Input<'grm>>()
                    .as_str(_src);
                let v = *reduce_expr(args[1], tc_env, allocs).into_value::<UnionIndex>();
                let b = *reduce_expr(args[2], tc_env, allocs).into_value::<UnionIndex>();
                PrismExpr::FnType(name, v, b)
            }
            "FnConstruct" => {
                assert_eq!(args.len(), 2);
                let name = reduce_expr(args[0], tc_env, allocs)
                    .into_value::<Input<'grm>>()
                    .as_str(_src);
                let b = *reduce_expr(args[1], tc_env, allocs).into_value::<UnionIndex>();
                PrismExpr::FnConstruct(name, b)
            }
            "FnDestruct" => {
                assert_eq!(args.len(), 2);
                let f = *reduce_expr(args[0], tc_env, allocs).into_value::<UnionIndex>();
                let v = *reduce_expr(args[1], tc_env, allocs).into_value::<UnionIndex>();
                PrismExpr::FnDestruct(f, v)
            }
            "TypeAssert" => {
                assert_eq!(args.len(), 2);

                let e = *reduce_expr(args[0], tc_env, allocs).into_value::<UnionIndex>();
                let typ = *reduce_expr(args[1], tc_env, allocs).into_value::<UnionIndex>();
                PrismExpr::TypeAssert(e, typ)
            }
            "GrammarDefine" => {
                assert_eq!(args.len(), 2);
                let b = *reduce_expr(args[0], tc_env, allocs).into_value::<UnionIndex>();
                let g = *reduce_expr(args[1], tc_env, allocs).into_value::<Guid>();

                PrismExpr::ShiftPoint(b, g)
            }
            _ => unreachable!(),
        };

        tc_env.store_from_source(expr, span)
    }
}

pub fn reduce_expr<'arn, 'grm: 'arn>(
    parsed: Parsed<'arn, 'grm>,
    tc_env: &mut PrismEnv,
    allocs: Allocs<'arn>,
) -> Parsed<'arn, 'grm> {
    if let Some(v) = parsed.try_into_value::<EnvCapture>() {
        let value = v.value.into_value::<ScopeEnter<'arn, 'grm>>();
        let env = v.env;
        let expr = *reduce_expr(value.0, tc_env, allocs).into_value::<UnionIndex>();
        let guid = value.1;

        let expr = tc_env.store_from_source(
            PrismExpr::ShiftTo(expr, guid),
            tc_env.value_origins[*expr].to_source_span(),
        );
        Parsed::from_value(allocs.alloc(expr))
    } else {
        parsed
    }
}

#[derive(Copy, Clone)]
pub struct ScopeEnter<'arn, 'grm>(Parsed<'arn, 'grm>, Guid);
impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for ScopeEnter<'arn, 'grm> {}
impl<'arn, 'grm: 'arn> Parsable<'arn, 'grm, PrismEnv<'arn, 'grm>> for ScopeEnter<'arn, 'grm> {
    fn from_construct(
        span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        allocs: Allocs<'arn>,
        src: &'grm str,
        tc_env: &mut PrismEnv,
    ) -> Self {
        assert_eq!(constructor, "Enter");
        ScopeEnter(args[0], *args[1].into_value())
    }
}
