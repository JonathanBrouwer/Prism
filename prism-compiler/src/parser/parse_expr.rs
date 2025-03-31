use crate::lang::env::{DbEnv, EnvEntry};
use crate::lang::{CorePrismExpr, PrismEnv};
use crate::parser::named_env::NamedEnv;
use crate::parser::{ParsedIndex, ParsedPrismExpr};
use prism_parser::core::input::Input;
use prism_parser::core::input_table::InputTable;
use prism_parser::core::span::Span;
use prism_parser::env::GenericEnv;
use prism_parser::grammar::grammar_file::GrammarFile;
use prism_parser::grammar::identifier::{Identifier, parse_identifier};
use prism_parser::parsable::Parsable;
use prism_parser::parsable::parsed::{ArcExt, Parsed};
use prism_parser::parser::VarMap;
use prism_parser::parser::placeholder_store::{ParsedPlaceholder, PlaceholderStore};
use std::sync::Arc;

pub type PrismEvalCtx = GenericEnv<ParsedPlaceholder, Option<ParsedPlaceholder>>;

pub fn eval_ctx_to_envs(
    env: &PrismEvalCtx,
    placeholders: &PlaceholderStore<PrismEnv>,
    input: &InputTable,
    prism_env: &mut PrismEnv,
) -> (NamedEnv, DbEnv) {
    match env.split() {
        None => (NamedEnv::default(), DbEnv::default()),
        Some(((key, value), rest)) => {
            let (named_env, db_env) = eval_ctx_to_envs(&rest, placeholders, input, prism_env);

            // Create dummy env entries, so that environments are safely reusable after the placeholders are filled in
            let dummy_named_env = named_env.insert_name(Identifier::from_const("_"), input);
            let dummy_db_env = db_env.cons(EnvEntry::RType(prism_env.new_tc_id()));

            // If the name or value of this entry is not known, continue
            let Some(key) = placeholders.get(*key) else {
                return (dummy_named_env, dummy_db_env);
            };
            let key = parse_identifier(key);

            // TODO we should also handle Nones here
            let Some(value) = value else {
                return (dummy_named_env, dummy_db_env);
            };
            let Some(value) = placeholders.get(*value) else {
                return (dummy_named_env, dummy_db_env);
            };
            let value = *value.value_ref::<ParsedIndex>();
            let value =
                prism_env.parsed_to_checked_with_env(value, &named_env, &mut Default::default());

            let named_env = named_env.insert_name(key, input);
            let db_env = db_env.cons(EnvEntry::RSubst(value, db_env.clone()));
            (named_env, db_env)
        }
    }
}

impl Parsable<PrismEnv> for ParsedIndex {
    type EvalCtx = PrismEvalCtx;

    fn from_construct(
        span: Span,
        constructor: Identifier,
        args: &[Parsed],
        src: &InputTable,
        prism_env: &mut PrismEnv,
    ) -> Self {
        let expr: ParsedPrismExpr = match constructor.as_str(src) {
            "Type" => {
                assert_eq!(args.len(), 0);

                ParsedPrismExpr::Type
            }
            "Name" => {
                assert_eq!(args.len(), 1);
                let name = parse_identifier(&args[0]);
                if name.as_str(&prism_env.input) == "_" {
                    ParsedPrismExpr::Free
                } else {
                    ParsedPrismExpr::Name(name)
                }
            }
            "Let" => {
                assert_eq!(args.len(), 3);
                let name = parse_identifier(&args[0]);
                let v = *args[1].value_ref::<ParsedIndex>();
                let b = *args[2].value_ref::<ParsedIndex>();
                ParsedPrismExpr::Let(name, v, b)
            }
            "FnType" => {
                assert_eq!(args.len(), 3);
                let name = parse_identifier(&args[0]);
                let v = *args[1].value_ref::<ParsedIndex>();
                let b = *args[2].value_ref::<ParsedIndex>();
                ParsedPrismExpr::FnType(name, v, b)
            }
            "FnConstruct" => {
                assert_eq!(args.len(), 2);
                let name = parse_identifier(&args[0]);
                let b = *args[1].value_ref::<ParsedIndex>();
                ParsedPrismExpr::FnConstruct(name, b)
            }
            "FnDestruct" => {
                assert_eq!(args.len(), 2);
                let f = *args[0].value_ref::<ParsedIndex>();
                let v = *args[1].value_ref::<ParsedIndex>();
                ParsedPrismExpr::FnDestruct(f, v)
            }
            "TypeAssert" => {
                assert_eq!(args.len(), 2);

                let e = *args[0].value_ref::<ParsedIndex>();
                let typ = *args[1].value_ref::<ParsedIndex>();
                ParsedPrismExpr::TypeAssert(e, typ)
            }
            "GrammarValue" => {
                assert_eq!(args.len(), 1);
                let grammar = args[0].value_cloned();
                ParsedPrismExpr::GrammarValue(grammar)
            }
            "GrammarType" => {
                assert_eq!(args.len(), 0);
                ParsedPrismExpr::GrammarType
            }
            "EnvCapture" => {
                assert_eq!(args.len(), 2);
                let value = args[0].value_ref::<EnvWrapper>();
                let captured_env = args[1].value_ref::<VarMap>().clone();

                let expr = *value.0.value_ref::<ParsedIndex>();

                ParsedPrismExpr::ShiftTo {
                    expr,
                    captured_env: captured_env,
                    adapt_env_len: value.1,
                    grammar: value.2.clone(),
                }
            }
            "Include" => {
                assert_eq!(args.len(), 1);
                let name = parse_identifier(&args[0]);

                let current_file = span.start_pos().file();

                let mut path = prism_env.input.get_path(current_file);
                assert!(path.pop());
                path.push(format!("{}.pr", name.as_str(&prism_env.input)));

                let next_file = prism_env.load_file(path);

                //TODO properly do errors
                let processed_file = prism_env.process_file(next_file);

                ParsedPrismExpr::Include(name, processed_file.core)
            }
            _ => unreachable!(),
        };

        prism_env.store_from_source(expr, span)
    }

    fn create_eval_ctx(
        constructor: Identifier,
        parent_ctx: &Self::EvalCtx,
        args: &[ParsedPlaceholder],
        src: &InputTable,
        _env: &mut PrismEnv,
    ) -> impl Iterator<Item = Option<Self::EvalCtx>> {
        match constructor.as_str(src) {
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
                    Some(parent_ctx.clone()),
                    Some(parent_ctx.insert(args[0], Some(args[1]))),
                ]
            }
            "FnType" => {
                assert_eq!(args.len(), 3);
                vec![None, Some(parent_ctx.clone()), Some(parent_ctx.clone())]
            }
            "FnConstruct" => {
                assert_eq!(args.len(), 2);
                vec![None, Some(parent_ctx.clone())]
            }
            "FnDestruct" => {
                assert_eq!(args.len(), 2);
                vec![Some(parent_ctx.clone()), Some(parent_ctx.clone())]
            }
            "TypeAssert" => {
                assert_eq!(args.len(), 2);
                vec![Some(parent_ctx.clone()), Some(parent_ctx.clone())]
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
        self: &Arc<ParsedIndex>,
        eval_ctx: &Self::EvalCtx,
        placeholders: &PlaceholderStore<PrismEnv>,
        src: &InputTable,
        prism_env: &mut PrismEnv,
    ) -> Arc<GrammarFile> {
        // Create context, ignore any errors that occur in this process
        let error_count = prism_env.errors.len();
        let (named_env, db_env) = eval_ctx_to_envs(eval_ctx, placeholders, src, prism_env);
        prism_env.errors.truncate(error_count);

        // Get original grammar function
        let original_e =
            prism_env.parsed_to_checked_with_env(**self, &named_env, &mut Default::default());
        let origin = prism_env.checked_origins[original_e.0];

        // Evaluate this to the grammar function
        let (grammar_fn_value, grammar_fn_env) = prism_env.beta_reduce_head(original_e, &db_env);

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
        let (reduced_value, _reduced_env) = prism_env.beta_reduce_head(e, &db_env);

        let CorePrismExpr::GrammarValue(grammar) = &prism_env.checked_values[reduced_value.0]
        else {
            panic!(
                "Tried to reduce expression which was not a grammar: {} / {} / {}",
                prism_env.parse_index_to_string(**self),
                prism_env.index_to_string(e),
                prism_env.index_to_string(reduced_value)
            )
        };

        // Insert the scope into the grammar, so we can find the scope again later in `reduce_expr`
        grammar.map_actions(&|e| {
            Arc::new(EnvWrapper(e.clone(), eval_ctx.len(), grammar.clone())).to_parsed()
        })
    }
}

#[derive(Clone)]
pub struct EnvWrapper(Parsed, usize, Arc<GrammarFile>);
