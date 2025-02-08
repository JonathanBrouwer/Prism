use crate::lang::PrismEnv;
use crate::parser::{ParsedIndex, ParsedPrismExpr};
use prism_parser::core::cache::Allocs;
use prism_parser::core::input::Input;
use prism_parser::core::span::Span;
use prism_parser::parsable::env_capture::EnvCapture;
use prism_parser::parsable::guid::Guid;
use prism_parser::parsable::parsed::Parsed;
use prism_parser::parsable::void::Void;
use prism_parser::parsable::{Parsable, ParseResult};
use prism_parser::parser::apply_action::ParsedPlaceholder;
use std::fmt::{Debug, Formatter};
use std::iter;

#[derive(Default, Copy, Clone)]
pub struct PrismEvalCtx<'arn>(Option<&'arn PrismEvalCtxNode<'arn>>);

impl<'arn> PrismEvalCtx<'arn> {
    pub fn get<'grm>(
        &self,
        k: &str,
        placeholders: &[Parsed<'arn, 'grm>],
        input: &'grm str,
    ) -> Option<Parsed<'arn, 'grm>> {
        let mut node = self.0?;
        loop {
            let key = placeholders[node.key.0];
            if key.try_into_value::<Void>().is_some() {
                node = node.next?;
                continue;
            }
            let key = key.into_value::<Input>().as_str(input);
            if key != k {
                node = node.next?;
                continue;
            }

            let Some(value) = node.value else {
                node = node.next?;
                continue;
            };
            let value = placeholders[value.0];
            if value.try_into_value::<Void>().is_some() {
                node = node.next?;
                continue;
            }
            return Some(value);
        }
    }

    #[must_use]
    pub fn insert(
        self,
        key: ParsedPlaceholder,
        value: Option<ParsedPlaceholder>,
        alloc: Allocs<'arn>,
    ) -> Self {
        Self(Some(alloc.alloc(PrismEvalCtxNode {
            next: self.0,
            key,
            value,
        })))
    }
}

#[derive(Copy, Clone)]
pub struct PrismEvalCtxNode<'arn> {
    next: Option<&'arn Self>,
    key: ParsedPlaceholder,
    value: Option<ParsedPlaceholder>,
}

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for PrismEvalCtx<'arn> {}

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for ParsedIndex {}
impl<'arn, 'grm: 'arn> Parsable<'arn, 'grm, PrismEnv<'arn, 'grm>> for ParsedIndex {
    type EvalCtx = PrismEvalCtx<'arn>;

    fn from_construct(
        span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        _allocs: Allocs<'arn>,
        _src: &'grm str,
        tc_env: &mut PrismEnv<'arn, 'grm>,
    ) -> Self {
        let expr: ParsedPrismExpr<'arn, 'grm> = match constructor {
            "Type" => {
                assert_eq!(args.len(), 0);

                ParsedPrismExpr::Type
            }
            "Name" => {
                assert_eq!(args.len(), 1);
                let name = reduce_expr(args[0], tc_env)
                    .into_value::<Input>()
                    .as_str(_src);
                if name == "_" {
                    ParsedPrismExpr::Free
                } else {
                    ParsedPrismExpr::Name(name)
                }
            }
            "Let" => {
                assert_eq!(args.len(), 3);
                let name = reduce_expr(args[0], tc_env)
                    .into_value::<Input<'grm>>()
                    .as_str(_src);
                let v = *reduce_expr(args[1], tc_env).into_value::<ParsedIndex>();
                let b = *reduce_expr(args[2], tc_env).into_value::<ParsedIndex>();
                ParsedPrismExpr::Let(name, v, b)
            }
            "FnType" => {
                assert_eq!(args.len(), 3);
                let name = reduce_expr(args[0], tc_env)
                    .into_value::<Input<'grm>>()
                    .as_str(_src);
                let v = *reduce_expr(args[1], tc_env).into_value::<ParsedIndex>();
                let b = *reduce_expr(args[2], tc_env).into_value::<ParsedIndex>();
                ParsedPrismExpr::FnType(name, v, b)
            }
            "FnConstruct" => {
                assert_eq!(args.len(), 2);
                let name = reduce_expr(args[0], tc_env)
                    .into_value::<Input<'grm>>()
                    .as_str(_src);
                let b = *reduce_expr(args[1], tc_env).into_value::<ParsedIndex>();
                ParsedPrismExpr::FnConstruct(name, b)
            }
            "FnDestruct" => {
                assert_eq!(args.len(), 2);
                let f = *reduce_expr(args[0], tc_env).into_value::<ParsedIndex>();
                let v = *reduce_expr(args[1], tc_env).into_value::<ParsedIndex>();
                ParsedPrismExpr::FnDestruct(f, v)
            }
            "TypeAssert" => {
                assert_eq!(args.len(), 2);

                let e = *reduce_expr(args[0], tc_env).into_value::<ParsedIndex>();
                let typ = *reduce_expr(args[1], tc_env).into_value::<ParsedIndex>();
                ParsedPrismExpr::TypeAssert(e, typ)
            }
            "GrammarDefine" => {
                assert_eq!(args.len(), 2);
                let b = *reduce_expr(args[0], tc_env).into_value::<ParsedIndex>();
                let g = *reduce_expr(args[1], tc_env).into_value::<Guid>();

                ParsedPrismExpr::ShiftLabel(b, g)
            }
            "ParserValue" => {
                assert_eq!(args.len(), 1);
                ParsedPrismExpr::ParserValue(args[0])
            }
            "ParsedType" => {
                assert_eq!(args.len(), 0);
                ParsedPrismExpr::ParsedType
            }
            _ => unreachable!(),
        };

        tc_env.store_from_source(expr, span)
    }

    fn create_eval_ctx(
        constructor: &'grm str,
        parent_ctx: Self::EvalCtx,
        args: &[ParsedPlaceholder],
        allocs: Allocs<'arn>,
        src: &'grm str,
        env: &mut PrismEnv<'arn, 'grm>,
    ) -> impl Iterator<Item = Option<Self::EvalCtx>> {
        match constructor {
            "Type" => {
                assert_eq!(args.len(), 0);
                vec![]
            }
            "Name" => {
                assert_eq!(args.len(), 1);
                vec![None]
            }
            "Let" => {
                assert_eq!(args.len(), 3);
                vec![
                    None,
                    Some(parent_ctx),
                    Some(parent_ctx.insert(args[0], Some(args[1]), allocs)),
                ]
            }
            "FnType" => {
                assert_eq!(args.len(), 3);
                vec![None, Some(parent_ctx), Some(parent_ctx)]
            }
            "FnConstruct" => {
                assert_eq!(args.len(), 2);
                vec![None, Some(parent_ctx)]
            }
            "FnDestruct" => {
                assert_eq!(args.len(), 2);
                vec![Some(parent_ctx), Some(parent_ctx)]
            }
            "TypeAssert" => {
                assert_eq!(args.len(), 2);
                vec![Some(parent_ctx), Some(parent_ctx)]
            }
            "GrammarDefine" => {
                assert_eq!(args.len(), 2);
                vec![None, Some(parent_ctx)]
            }
            "ParserValue" => {
                assert_eq!(args.len(), 1);
                vec![None]
            }
            "ParsedType" => {
                assert_eq!(args.len(), 0);
                vec![]
            }
            _ => unreachable!(),
        }
        .into_iter()
    }

    fn eval_to_parsed(
        &'arn self,
        eval_ctx: Self::EvalCtx,
        placeholders: &[Parsed<'arn, 'grm>],
        allocs: Allocs<'arn>,
        src: &'grm str,
        env: &mut PrismEnv<'arn, 'grm>,
    ) -> Parsed<'arn, 'grm> {
        //TODO convert eval_ctx to env and run

        let x = eval_ctx.get("test", placeholders, src);
        eprintln!("{x:?}");

        // eprintln!("{:?}", eval_ctx);
        self.to_parsed()
    }
}

pub fn reduce_expr<'arn, 'grm: 'arn>(
    parsed: Parsed<'arn, 'grm>,
    tc_env: &mut PrismEnv<'arn, 'grm>,
) -> Parsed<'arn, 'grm> {
    if let Some(v) = parsed.try_into_value::<EnvCapture>() {
        let value = v.value.into_value::<ScopeEnter<'arn, 'grm>>();
        let env = v.env;
        let expr = *reduce_expr(value.0, tc_env).into_value::<ParsedIndex>();
        let guid = value.1;

        let expr = tc_env.store_from_source(
            ParsedPrismExpr::ShiftTo(expr, guid, env),
            tc_env.parsed_spans[*expr],
        );
        Parsed::from_value(tc_env.allocs.alloc(expr))
    } else {
        parsed
    }
}

#[derive(Copy, Clone)]
pub struct ScopeEnter<'arn, 'grm>(Parsed<'arn, 'grm>, Guid);
impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for ScopeEnter<'arn, 'grm> {}
impl<'arn, 'grm: 'arn> Parsable<'arn, 'grm, PrismEnv<'arn, 'grm>> for ScopeEnter<'arn, 'grm> {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        _allocs: Allocs<'arn>,
        _src: &'grm str,
        _tc_env: &mut PrismEnv,
    ) -> Self {
        assert_eq!(constructor, "Enter");
        ScopeEnter(args[0], *args[1].into_value())
    }
}
