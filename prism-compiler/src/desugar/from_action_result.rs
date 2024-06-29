use std::env::var;
use prism_parser::parser::var_map::VarMap;
use crate::desugar::{Guid, ParseEnv, ParseIndex, SourceExpr};
use prism_parser::rule_action::action_result::ActionResult;

impl ParseEnv {
    pub fn insert_from_action_result(&mut self, value: &ActionResult, program: &str, vars: VarMap) -> ParseIndex {
        let ActionResult::Construct(span, constructor, args) = value else {
            unreachable!("Parsing an expression always returns a Construct");
        };
        let inner = match *constructor {
            "Type" => {
                assert_eq!(args.len(), 0);
                SourceExpr::Type
            }
            "Let" => {
                assert_eq!(args.len(), 3);
                SourceExpr::Let(
                    args[0].get_value(program).to_string(),
                    self.insert_from_action_result(&args[1], program, vars),
                    self.insert_from_action_result(&args[2], program, vars),
                )
            }
            "Variable" => {
                assert_eq!(args.len(), 1);
                let name = args[0].get_value(program).to_string();
                if let Some(value) = vars.get(&name) {
                    todo!()
                } else {
                    SourceExpr::Variable(name)
                }
            }
            "FnType" => {
                assert_eq!(args.len(), 3);
                SourceExpr::FnType(
                    args[0].get_value(program).to_string(),
                    self.insert_from_action_result(&args[1], program, vars),
                    self.insert_from_action_result(&args[2], program, vars),
                )
            }
            "FnConstruct" => {
                assert_eq!(args.len(), 3);
                SourceExpr::FnConstruct(
                    args[0].get_value(program).to_string(),
                    self.insert_from_action_result(&args[1], program, vars),
                    self.insert_from_action_result(&args[2], program, vars),
                )
            }
            "FnDestruct" => {
                assert_eq!(args.len(), 2);
                SourceExpr::FnDestruct(
                    self.insert_from_action_result(&args[0], program, vars),
                    self.insert_from_action_result(&args[1], program, vars),
                )
            }
            "ScopeStart" => {
                assert_eq!(args.len(), 2);
                SourceExpr::ScopeStart(
                    self.insert_from_action_result(&args[0], program, vars),
                    Self::parse_guid(&args[1]),
                )
            },
            "ScopeJump" => {
                assert_eq!(args.len(), 3);
                let ActionResult::Env(env) = args[2].as_ref() else {
                    unreachable!()
                };
                SourceExpr::ScopeJump(
                    self.insert_from_action_result(&args[0], program, *env),
                    Self::parse_guid(&args[1]),
                )
            },
            _ => unreachable!(),
        };
        self.store(inner, *span)
    }

    fn parse_guid(ar: &ActionResult) -> Guid {
        let ActionResult::Guid(v) = ar else {
            unreachable!()
        };
        Guid(*v)
    }
}
