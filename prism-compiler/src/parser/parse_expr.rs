use crate::lang::{PartialExpr, UnionIndex};
use crate::parser::parse_env::ParsedEnv;
use prism_parser::core::cache::Allocs;
use prism_parser::core::input::Input;
use prism_parser::core::span::Span;
use prism_parser::parsable::env_capture::EnvCapture;
use prism_parser::parsable::guid::Guid;
use prism_parser::parsable::parsed::Parsed;
use prism_parser::parsable::Parsable;

impl<'arn, 'grm: 'arn> Parsable<'arn, 'grm> for UnionIndex {
    fn from_construct(
        span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        allocs: Allocs<'arn>,
        src: &'grm str,
    ) -> Self {
        let env = args[0].into_value::<ParsedEnv<'arn>>();
        let args = &args[1..];

        let expr = match constructor {
            "Type" => {
                assert_eq!(args.len(), 0);

                PartialExpr::Type
            }
            "Name" => {
                assert_eq!(args.len(), 1);
                let name = reduce(args[0]).into_value::<Input>().as_str(src);

                let (idx, _) = env.get(name).unwrap();

                PartialExpr::DeBruijnIndex(idx)
            }
            "Let" => {
                assert_eq!(args.len(), 3);
                let _name = reduce(args[0]).into_value::<Input>().as_str(src);
                let v = *reduce(args[1]).into_value::<UnionIndex>();
                let b = *reduce(args[2]).into_value::<UnionIndex>();
                PartialExpr::Let(v, b)
            }
            // "FnType" => {
            //     assert_eq!(args.len(), 3);
            //     let name = reduce(args[0]).into_value::<Input>().as_str(src);
            //     let v = *reduce(args[1]).into_value::<UnionIndex>();
            //     let b = *reduce(args[2]).into_value::<UnionIndex>();
            //     PartialExpr::FnType(v, b)
            //
            //     // let name = Self::parse_name(&args[0], program);
            //     //
            //     // let v = self.insert_from_action_result_rec(&args[1], program, vars);
            //     // let b = self.insert_from_action_result_rec(
            //     //     &args[2],
            //     //     program,
            //     //     &vars.insert_name(name, program),
            //     // );
            //     //
            //     // PartialExpr::FnType(v, b)
            // }
            // "FnConstruct" => {
            //     assert_eq!(args.len(), 2);
            //     let name = reduce(args[0]).into_value::<Input>().as_str(src);
            //     let b = *reduce(args[1]).into_value::<UnionIndex>();
            //     PartialExpr::FnConstruct(b)
            //
            //     // let name = Self::parse_name(&args[0], program);
            //     //
            //     // let b = self.insert_from_action_result_rec(
            //     //     &args[1],
            //     //     program,
            //     //     &vars.insert_name(name, program),
            //     // );
            //     //
            //     // PartialExpr::FnConstruct(b)
            // }
            // "FnDestruct" => {
            //     assert_eq!(args.len(), 2);
            //     let f = *reduce(args[0]).into_value::<UnionIndex>();
            //     let v = *reduce(args[1]).into_value::<UnionIndex>();
            //     PartialExpr::FnDestruct(f, v)
            //
            //     // let f = self.insert_from_action_result_rec(&args[0], program, vars);
            //     // let v = self.insert_from_action_result_rec(&args[1], program, vars);
            //     //
            //     // PartialExpr::FnDestruct(f, v)
            // }
            // "GrammarDefine" => {
            //     assert_eq!(args.len(), 4);
            //     let guid = *reduce(args[1]).into_value::<Guid>();
            //     let _id = reduce(args[2]).into_value::<Input>().as_str(src);
            //     let _grammar = reduce(args[3]);
            //
            //     return *reduce(args[0]).into_value::<UnionIndex>();
            //
            //     // let guid = Self::parse_guid(&args[1]);
            //     // let _id = Self::parse_name(&args[2], program);
            //     // let _grammar = &args[3];
            //     //
            //     // return self.insert_from_action_result_rec(
            //     //     &args[0],
            //     //     program,
            //     //     &vars.insert_jump(guid),
            //     // );
            // }
            // "TypeAssert" => {
            //     assert_eq!(args.len(), 2);
            //
            //     let e = *reduce(args[0]).into_value::<UnionIndex>();
            //     let typ = *reduce(args[1]).into_value::<UnionIndex>();
            //     PartialExpr::TypeAssert(e, typ)
            //
            //     // let e = self.insert_from_action_result_rec(&args[0], program, vars);
            //     // let typ = self.insert_from_action_result_rec(&args[1], program, vars);
            //     //
            //     // PartialExpr::TypeAssert(e, typ)
            // }
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

        UnionIndex(usize::MAX)
    }
}

fn reduce<'arn, 'grm: 'arn>(parsed: Parsed<'arn, 'grm>) -> Parsed<'arn, 'grm> {
    //TODO
    parsed
    // if let Some(v) = parsed.try_into_value::<EnvCapture>() {
    //     let v = v.value.into_value::<ScopeEnter>();
    //     reduce(v.0)
    // } else {
    //     parsed
    // }
}

#[derive(Copy, Clone)]
pub struct ScopeEnter<'arn, 'grm>(Parsed<'arn, 'grm>, Guid);
impl<'arn, 'grm: 'arn> Parsable<'arn, 'grm> for ScopeEnter<'arn, 'grm> {
    fn from_construct(
        span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        allocs: Allocs<'arn>,
        src: &'grm str,
    ) -> Self {
        ScopeEnter(args[0], *args[1].into_value::<Guid>())
    }
}
