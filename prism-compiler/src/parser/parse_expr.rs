use crate::lang::env::{DbEnv, EnvEntry};
use crate::lang::{CheckedPrismExpr, PrismEnv};
use crate::parser::named_env::NamedEnv;
use crate::parser::{ParsedIndex, ParsedPrismExpr};
use prism_parser::core::cache::Allocs;
use prism_parser::core::input::Input;
use prism_parser::core::span::Span;
use prism_parser::grammar::grammar_file::GrammarFile;
use prism_parser::parsable::env_capture::EnvCapture;
use prism_parser::parsable::guid::Guid;
use prism_parser::parsable::parsed::Parsed;
use prism_parser::parsable::{Parsable, ParseResult};
use prism_parser::parser::placeholder_store::{ParsedPlaceholder, PlaceholderStore};
use std::collections::HashMap;

#[derive(Default, Copy, Clone)]
pub struct PrismEvalCtx<'arn>(Option<&'arn PrismEvalCtxNode<'arn>>);

impl<'arn> PrismEvalCtx<'arn> {
    pub fn to_envs<'grm>(
        &self,
        placeholders: &PlaceholderStore<'arn, 'grm, PrismEnv<'arn, 'grm>>,
        input: &'grm str,
        prism_env: &mut PrismEnv<'arn, 'grm>,
    ) -> (NamedEnv<'arn, 'grm>, DbEnv) {
        match self.0 {
            None => (NamedEnv::default(), DbEnv::default()),
            Some(node) => {
                let (named_env, db_env) =
                    PrismEvalCtx(node.next).to_envs(placeholders, input, prism_env);

                // Create dummy env entries, so that environments are safely reusable after the placeholders are filled in
                let dummy_named_env = named_env.insert_name("_", input);
                let dummy_db_env = db_env.cons(EnvEntry::RType(prism_env.new_tc_id()));

                // If the name or value of this entry is not known, continue
                let Some(key) = placeholders.get(node.key) else {
                    return (dummy_named_env, dummy_db_env);
                };
                let key = key.into_value::<Input>().as_str(input);

                // TODO we should also handle Nones here
                let Some(value) = node.value else {
                    return (dummy_named_env, dummy_db_env);
                };
                let Some(value) = placeholders.get(value) else {
                    return (dummy_named_env, dummy_db_env);
                };
                let value = value.into_value::<ParsedIndex>();
                let value =
                    prism_env.parsed_to_checked_with_env(*value, &named_env, &mut HashMap::new());

                let named_env = named_env.insert_name(key, input);
                let db_env = db_env.cons(EnvEntry::RSubst(value, db_env.clone()));
                (named_env, db_env)
            }
        }
    }

    /// Insert a value into this ctx
    /// `value` is None of this is a `type` entry rather than a `subst` entry
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
        prism_env: &mut PrismEnv<'arn, 'grm>,
    ) -> Self {
        let expr: ParsedPrismExpr<'arn, 'grm> = match constructor {
            "Type" => {
                assert_eq!(args.len(), 0);

                ParsedPrismExpr::Type
            }
            "Name" => {
                assert_eq!(args.len(), 1);
                let name = reduce_expr(args[0], prism_env)
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
                let name = reduce_expr(args[0], prism_env)
                    .into_value::<Input<'grm>>()
                    .as_str(_src);
                let v = *reduce_expr(args[1], prism_env).into_value::<ParsedIndex>();
                let b = *reduce_expr(args[2], prism_env).into_value::<ParsedIndex>();
                ParsedPrismExpr::Let(name, v, b)
            }
            "FnType" => {
                assert_eq!(args.len(), 3);
                let name = reduce_expr(args[0], prism_env)
                    .into_value::<Input<'grm>>()
                    .as_str(_src);
                let v = *reduce_expr(args[1], prism_env).into_value::<ParsedIndex>();
                let b = *reduce_expr(args[2], prism_env).into_value::<ParsedIndex>();
                ParsedPrismExpr::FnType(name, v, b)
            }
            "FnConstruct" => {
                assert_eq!(args.len(), 2);
                let name = reduce_expr(args[0], prism_env)
                    .into_value::<Input<'grm>>()
                    .as_str(_src);
                let b = *reduce_expr(args[1], prism_env).into_value::<ParsedIndex>();
                ParsedPrismExpr::FnConstruct(name, b)
            }
            "FnDestruct" => {
                assert_eq!(args.len(), 2);
                let f = *reduce_expr(args[0], prism_env).into_value::<ParsedIndex>();
                let v = *reduce_expr(args[1], prism_env).into_value::<ParsedIndex>();
                ParsedPrismExpr::FnDestruct(f, v)
            }
            "TypeAssert" => {
                assert_eq!(args.len(), 2);

                let e = *reduce_expr(args[0], prism_env).into_value::<ParsedIndex>();
                let typ = *reduce_expr(args[1], prism_env).into_value::<ParsedIndex>();
                ParsedPrismExpr::TypeAssert(e, typ)
            }
            "GrammarValue" => {
                assert_eq!(args.len(), 2);
                let grammar = args[0].into_value();
                let guid: Guid = *args[1].into_value();

                ParsedPrismExpr::GrammarValue(grammar, guid)
            }
            "GrammarType" => {
                assert_eq!(args.len(), 0);
                ParsedPrismExpr::GrammarType
            }
            _ => unreachable!(),
        };

        prism_env.store_from_source(expr, span)
    }

    fn create_eval_ctx(
        constructor: &'grm str,
        parent_ctx: Self::EvalCtx,
        args: &[ParsedPlaceholder],
        allocs: Allocs<'arn>,
        _src: &'grm str,
        _env: &mut PrismEnv<'arn, 'grm>,
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
            "GrammarValue" => {
                assert_eq!(args.len(), 2);
                vec![None, None]
            }
            "GrammarType" => {
                assert_eq!(args.len(), 0);
                vec![]
            }
            _ => unreachable!(),
        }
        .into_iter()
    }

    fn eval_to_grammar(
        &'arn self,
        eval_ctx: Self::EvalCtx,
        placeholders: &PlaceholderStore<'arn, 'grm, PrismEnv<'arn, 'grm>>,
        _allocs: Allocs<'arn>,
        src: &'grm str,
        prism_env: &mut PrismEnv<'arn, 'grm>,
    ) -> &'arn GrammarFile<'arn, 'grm> {
        // Create context, ignore any errors that occur in this process
        let error_count = prism_env.errors.len();
        let (named_env, db_env) = eval_ctx.to_envs(placeholders, src, prism_env);
        prism_env.errors.truncate(error_count);

        let value = prism_env.parsed_to_checked_with_env(*self, &named_env, &mut HashMap::new());
        let (reduced_value, reduced_env) = prism_env.beta_reduce_head(value, db_env.clone());

        let common_index = db_env
            .iter()
            .rev()
            .zip(reduced_env.iter().rev())
            .take_while(|(a, b)| a == b)
            .count();
        assert_eq!(common_index, reduced_env.len());

        let CheckedPrismExpr::GrammarValue(grammar, guid) =
            prism_env.checked_values[reduced_value.0]
        else {
            panic!(
                "Tried to reduce expression which was not a grammar: {} / {} / {}",
                prism_env.parse_index_to_string(*self),
                prism_env.index_to_string(value),
                prism_env.index_to_string(reduced_value)
            )
        };

        prism_env.grammar_envs.insert(
            guid,
            GrammarEnvEntry {
                env: reduced_env,
                common_index,
            },
        );

        grammar
    }
}

#[derive(Clone)]
pub struct GrammarEnvEntry {
    env: DbEnv,
    common_index: usize,
}

pub fn reduce_expr<'arn, 'grm: 'arn>(
    parsed: Parsed<'arn, 'grm>,
    prism_env: &mut PrismEnv<'arn, 'grm>,
) -> Parsed<'arn, 'grm> {
    if let Some(v) = parsed.try_into_value::<EnvCapture>() {
        let value = v.value.into_value::<ScopeEnter<'arn, 'grm>>();
        let env = v.env;
        let expr = *reduce_expr(value.0, prism_env).into_value::<ParsedIndex>();
        let guid = value.1;

        let grammar_env_entry = prism_env.grammar_envs.get(&guid).unwrap();

        let expr = prism_env.store_from_source(
            ParsedPrismExpr::ShiftTo(expr, guid, env),
            prism_env.parsed_spans[*expr],
        );
        Parsed::from_value(prism_env.allocs.alloc(expr))
    } else {
        parsed
    }
}

#[derive(Copy, Clone)]
pub struct ScopeEnter<'arn, 'grm>(Parsed<'arn, 'grm>, Guid);
impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for ScopeEnter<'arn, 'grm> {}
impl<'arn, 'grm: 'arn> Parsable<'arn, 'grm, PrismEnv<'arn, 'grm>> for ScopeEnter<'arn, 'grm> {
    type EvalCtx = PrismEvalCtx<'arn>;

    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        _allocs: Allocs<'arn>,
        _src: &'grm str,
        _prism_env: &mut PrismEnv,
    ) -> Self {
        assert_eq!(constructor, "Enter");
        ScopeEnter(args[0], *args[1].into_value())
    }

    fn create_eval_ctx(
        _constructor: &'grm str,
        parent_ctx: Self::EvalCtx,
        _arg_placeholders: &[ParsedPlaceholder],
        _allocs: Allocs<'arn>,
        _src: &'grm str,
        _env: &mut PrismEnv<'arn, 'grm>,
    ) -> impl Iterator<Item = Option<Self::EvalCtx>> {
        [Some(parent_ctx), None].into_iter()
    }
}
