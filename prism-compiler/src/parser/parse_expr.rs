use crate::lang::env::{DbEnv, EnvEntry};
use crate::lang::{CorePrismExpr, PrismEnv};
use crate::parser::named_env::NamedEnv;
use crate::parser::{ParsedIndex, ParsedPrismExpr};
use prism_parser::core::allocs::Allocs;
use prism_parser::core::input::Input;
use prism_parser::core::input_table::InputTable;
use prism_parser::core::span::Span;
use prism_parser::env::GenericEnv;
use prism_parser::grammar::grammar_file::GrammarFile;
use prism_parser::parsable::parsed::Parsed;
use prism_parser::parsable::{Parsable, ParseResult};
use prism_parser::parser::VarMap;
use prism_parser::parser::placeholder_store::{ParsedPlaceholder, PlaceholderStore};

pub type PrismEvalCtx<'arn> = GenericEnv<'arn, ParsedPlaceholder, Option<ParsedPlaceholder>>;

pub fn eval_ctx_to_envs<'arn>(
    env: PrismEvalCtx<'arn>,
    placeholders: &PlaceholderStore<'arn, PrismEnv<'arn>>,
    input: &InputTable<'arn>,
    prism_env: &mut PrismEnv<'arn>,
) -> (NamedEnv<'arn>, DbEnv<'arn>) {
    match env.split() {
        None => (NamedEnv::default(), DbEnv::default()),
        Some(((key, value), rest)) => {
            let (named_env, db_env) = eval_ctx_to_envs(rest, placeholders, input, prism_env);

            // Create dummy env entries, so that environments are safely reusable after the placeholders are filled in
            let dummy_named_env = named_env.insert_name("_", input, prism_env.allocs);
            let dummy_db_env =
                db_env.cons(EnvEntry::RType(prism_env.new_tc_id()), prism_env.allocs);

            // If the name or value of this entry is not known, continue
            let Some(key) = placeholders.get(key) else {
                return (dummy_named_env, dummy_db_env);
            };
            let key = key.into_value::<Input>().as_str(input);

            // TODO we should also handle Nones here
            let Some(value) = value else {
                return (dummy_named_env, dummy_db_env);
            };
            let Some(value) = placeholders.get(value) else {
                return (dummy_named_env, dummy_db_env);
            };
            let value = value.into_value::<ParsedIndex>();
            let value =
                prism_env.parsed_to_checked_with_env(*value, named_env, &mut Default::default());

            let named_env = named_env.insert_name(key, input, prism_env.allocs);
            let db_env = db_env.cons(EnvEntry::RSubst(value, db_env), prism_env.allocs);
            (named_env, db_env)
        }
    }
}

impl ParseResult for ParsedIndex {}
impl<'arn> Parsable<'arn, PrismEnv<'arn>> for ParsedIndex {
    type EvalCtx = PrismEvalCtx<'arn>;

    fn from_construct(
        span: Span,
        constructor: Identifier,
        args: &[Parsed<'arn>],
        _allocs: Allocs<'arn>,
        src: &InputTable<'arn>,
        prism_env: &mut PrismEnv<'arn>,
    ) -> Self {
        let expr: ParsedPrismExpr<'arn> = match constructor {
            "Type" => {
                assert_eq!(args.len(), 0);

                ParsedPrismExpr::Type
            }
            "Name" => {
                assert_eq!(args.len(), 1);
                let name = args[0].into_value::<Input>().as_str(src);
                if name == "_" {
                    ParsedPrismExpr::Free
                } else {
                    ParsedPrismExpr::Name(name)
                }
            }
            "Let" => {
                assert_eq!(args.len(), 3);
                let name = args[0].into_value::<Input>().as_str(src);
                let v = *args[1].into_value::<ParsedIndex>();
                let b = *args[2].into_value::<ParsedIndex>();
                ParsedPrismExpr::Let(name, v, b)
            }
            "FnType" => {
                assert_eq!(args.len(), 3);
                let name = args[0].into_value::<Input>().as_str(src);
                let v = *args[1].into_value::<ParsedIndex>();
                let b = *args[2].into_value::<ParsedIndex>();
                ParsedPrismExpr::FnType(name, v, b)
            }
            "FnConstruct" => {
                assert_eq!(args.len(), 2);
                let name = args[0].into_value::<Input>().as_str(src);
                let b = *args[1].into_value::<ParsedIndex>();
                ParsedPrismExpr::FnConstruct(name, b)
            }
            "FnDestruct" => {
                assert_eq!(args.len(), 2);
                let f = *args[0].into_value::<ParsedIndex>();
                let v = *args[1].into_value::<ParsedIndex>();
                ParsedPrismExpr::FnDestruct(f, v)
            }
            "TypeAssert" => {
                assert_eq!(args.len(), 2);

                let e = *args[0].into_value::<ParsedIndex>();
                let typ = *args[1].into_value::<ParsedIndex>();
                ParsedPrismExpr::TypeAssert(e, typ)
            }
            "GrammarValue" => {
                assert_eq!(args.len(), 1);
                let grammar = args[0].into_value();
                ParsedPrismExpr::GrammarValue(grammar)
            }
            "GrammarType" => {
                assert_eq!(args.len(), 0);
                ParsedPrismExpr::GrammarType
            }
            "EnvCapture" => {
                assert_eq!(args.len(), 2);
                let value = args[0].into_value::<EnvWrapper>();
                let captured_env = args[1].into_value::<VarMap<'arn>>();

                let expr = *value.0.into_value::<ParsedIndex>();

                ParsedPrismExpr::ShiftTo {
                    expr,
                    captured_env: *captured_env,
                    adapt_env_len: value.1,
                    grammar: value.2,
                }
            }
            "Include" => {
                assert_eq!(args.len(), 1);
                let n = args[0].into_value::<Input>().as_str(src);

                let current_file = span.start_pos().file();

                let mut path = prism_env.input.get_path(current_file);
                assert!(path.pop());
                path.push(format!("{n}.pr"));

                let next_file = prism_env.load_file(path);

                //TODO properly do errors
                let processed_file = prism_env.process_file(next_file);

                ParsedPrismExpr::Include(n, processed_file.core)
            }
            _ => unreachable!(),
        };

        prism_env.store_from_source(expr, span)
    }

    fn create_eval_ctx(
        constructor: Identifier,
        parent_ctx: Self::EvalCtx,
        args: &[ParsedPlaceholder],
        allocs: Allocs<'arn>,
        _src: &InputTable<'arn>,
        _env: &mut PrismEnv<'arn>,
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
                assert_eq!(args.len(), 1);
                vec![None]
            }
            "GrammarType" => {
                assert_eq!(args.len(), 0);
                vec![]
            }
            "Include" => {
                assert_eq!(args.len(), 1);
                vec![None]
            }
            _ => unreachable!(),
        }
        .into_iter()
    }

    fn eval_to_grammar(
        &'arn self,
        eval_ctx: Self::EvalCtx,
        placeholders: &PlaceholderStore<'arn, PrismEnv<'arn>>,
        src: &InputTable<'arn>,
        prism_env: &mut PrismEnv<'arn>,
    ) -> &'arn GrammarFile<'arn> {
        // Create context, ignore any errors that occur in this process
        let error_count = prism_env.errors.len();
        let (named_env, db_env) = eval_ctx_to_envs(eval_ctx, placeholders, src, prism_env);
        prism_env.errors.truncate(error_count);

        // Get original grammar function
        let original_e =
            prism_env.parsed_to_checked_with_env(*self, named_env, &mut Default::default());
        let origin = prism_env.checked_origins[original_e.0];

        // Evaluate this to the grammar function
        let (grammar_fn_value, grammar_fn_env) = prism_env.beta_reduce_head(original_e, db_env);

        // Create expression that takes first element from this function
        let e = prism_env.store_checked(CorePrismExpr::DeBruijnIndex(0), origin);
        let mut e = prism_env.store_checked(CorePrismExpr::FnConstruct(e), origin);
        for _ in 0..grammar_fn_env.len() {
            e = prism_env.store_checked(CorePrismExpr::FnConstruct(e), origin);
        }
        let free_returntype = prism_env.store_checked(CorePrismExpr::Free, origin);
        let grammar_fn_value = prism_env.store_checked(
            CorePrismExpr::FnDestruct(grammar_fn_value, free_returntype),
            origin,
        );
        let e = prism_env.store_checked(CorePrismExpr::FnDestruct(grammar_fn_value, e), origin);

        // Evaluate this further
        let (reduced_value, _reduced_env) = prism_env.beta_reduce_head(e, db_env);

        let CorePrismExpr::GrammarValue(grammar) = prism_env.checked_values[reduced_value.0] else {
            panic!(
                "Tried to reduce expression which was not a grammar: {} / {} / {}",
                prism_env.parse_index_to_string(*self),
                prism_env.index_to_string(e),
                prism_env.index_to_string(reduced_value)
            )
        };

        // Insert the scope into the grammar, so we can find the scope again later in `reduce_expr`
        grammar.map_actions(
            &|e| {
                prism_env
                    .allocs
                    .alloc(EnvWrapper(e, eval_ctx.len(), grammar))
                    .to_parsed()
            },
            prism_env.allocs,
        )
    }
}

#[derive(Copy, Clone)]
pub struct EnvWrapper<'arn>(Parsed<'arn>, usize, &'arn GrammarFile<'arn>);
impl ParseResult for EnvWrapper<'_> {}
