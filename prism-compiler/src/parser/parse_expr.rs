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
impl<'arn, 'grm: 'arn> Parsable2<'arn, 'grm, TcEnv> for UnionIndex {
    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        _args: &[Parsed<'arn, 'grm>],
        allocs: Allocs<'arn>,
        _src: &'grm str,
        tc_env: &mut TcEnv,
    ) -> Result<Self, String> {
        let env = _args[0].into_value::<ParsedEnv<'arn>>();
        let args = &_args[1..];

        let expr = match constructor {
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
                    let (idx, _) = env
                        .get(name)
                        .ok_or_else(|| format!("Failed to find {name} in scope"))?;
                    PartialExpr::DeBruijnIndex(idx)
                }
            }
            "Let" => {
                assert_eq!(args.len(), 3);
                let _name = reduce_expr(args[0], tc_env, allocs)
                    .into_value::<Input>()
                    .as_str(_src);
                let v = *reduce_expr(args[1], tc_env, allocs).into_value::<UnionIndex>();
                let b = *reduce_expr(args[2], tc_env, allocs).into_value::<UnionIndex>();
                PartialExpr::Let(v, b)
            }
            "FnType" => {
                assert_eq!(args.len(), 3);
                let _name = reduce_expr(args[0], tc_env, allocs)
                    .into_value::<Input>()
                    .as_str(_src);
                let v = *reduce_expr(args[1], tc_env, allocs).into_value::<UnionIndex>();
                let b = *reduce_expr(args[2], tc_env, allocs).into_value::<UnionIndex>();
                PartialExpr::FnType(v, b)
            }
            "FnConstruct" => {
                assert_eq!(args.len(), 2);
                let _name = reduce_expr(args[0], tc_env, allocs)
                    .into_value::<Input>()
                    .as_str(_src);
                let b = *reduce_expr(args[1], tc_env, allocs).into_value::<UnionIndex>();
                PartialExpr::FnConstruct(b)
            }
            "FnDestruct" => {
                assert_eq!(args.len(), 2);
                let f = *reduce_expr(args[0], tc_env, allocs).into_value::<UnionIndex>();
                let v = *reduce_expr(args[1], tc_env, allocs).into_value::<UnionIndex>();
                PartialExpr::FnDestruct(f, v)
            }
            // "GrammarDefine" => {
            //     assert_eq!(args.len(), 4);
            //     let guid = *reduce(args[1]).into_value::<Guid>();
            //     let _id = reduce(args[2]).into_value::<Input>().as_str(src);
            //     let _grammar = reduce(args[3]);
            //
            //     return *reduce(args[0]).into_value::<UnionIndex>();
            //
            //     // return self.insert_from_action_result_rec(
            //     //     &args[0],
            //     //     program,
            //     //     &vars.insert_jump(guid),
            //     // );
            // }
            "TypeAssert" => {
                assert_eq!(args.len(), 2);

                let e = *reduce_expr(args[0], tc_env, allocs).into_value::<UnionIndex>();
                let typ = *reduce_expr(args[1], tc_env, allocs).into_value::<UnionIndex>();
                PartialExpr::TypeAssert(e, typ)
            }
            // "Name" => {
            //     let name = reduce(args[0]).into_value::<Input>().as_str(src);
            //     PartialExpr::DeBruijnIndex(0)
            //
            //     //     //             let e = if name == "_" {
            //     //     //                 PartialExpr::Free
            //     //     //             } else {
            //     //     //                 match vars.get(name) {
            //     //     //                     None => {
            //     //     //                         self.errors.push(TypeError::UnknownName(*span));
            //     //     //                         PartialExpr::Free
            //     //     //                     }
            //     //     //                     Some(ScopeValue::FromGrammar(ar, scope_vars)) => {
            //     //     //                         // Create a new scope based on the current depth and `scope_vars`
            //     //     //                         let mut scope_vars_with_hygienic_decls = Scope {
            //     //     //                             depth: vars.depth,
            //     //     //                             ..scope_vars.clone()
            //     //     //                         };
            //     //     //
            //     //     //                         // Insert hygienically declared variables into the scope
            //     //     //                         for (k, v) in &vars.hygienic_decls {
            //     //     //                             scope_vars_with_hygienic_decls =
            //     //     //                                 scope_vars_with_hygienic_decls.insert_name_at(k, *v, program);
            //     //     //                         }
            //     //     //
            //     //     //                         // Parse the value in the new scope
            //     //     //                         return self.insert_from_action_result_rec(
            //     //     //                             ar,
            //     //     //                             program,
            //     //     //                             &scope_vars_with_hygienic_decls,
            //     //     //                         );
            //     //     //                     }
            //     //     //                     Some(ScopeValue::FromEnv(ix)) => {
            //     //     //                         PartialExpr::DeBruijnIndex(vars.depth - ix - 1)
            //     //     //                     }
            //     //     //                 }
            //     //     //             };
            //     //     //             (e, *span)
            // }
            _ => unreachable!(),
        };

        //     //         ActionResult::Value(span) => {
        //     //             let name = Self::parse_name(value, program);
        //     //

        //     //         }
        //     //         ActionResult::WithEnv(new_vars, ar) => {
        //     //             let ActionResult::Construct(_span, "ScopeEnter", args) = ar else {
        //     //                 unreachable!()
        //     //             };
        //     //             let guid = Self::parse_guid(&args[1]);
        //     //             let vars = vars.jump(guid).extend_with_ars(new_vars, vars);
        //     //
        //     //             return self.insert_from_action_result_rec(&args[0], program, &vars);
        //     //         }

        Ok(tc_env.store_from_source(expr, _span))
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
impl<'arn, 'grm: 'arn> Parsable2<'arn, 'grm, TcEnv> for ScopeEnter<'arn, 'grm> {
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
