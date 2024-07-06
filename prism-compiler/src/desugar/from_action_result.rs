use prism_parser::parser::parser_instance::Arena;
use crate::desugar::{Guid, ParseEnv, ParseIndex, SourceExpr};
use prism_parser::parser::var_map::{VarMap, VarMapNode};
use prism_parser::rule_action::action_result::ActionResult;

impl ParseEnv {
    pub fn insert_from_action_result<'arn, 'grm>(
        &mut self,
        value: &ActionResult<'arn, 'grm>,
        program: &str,
        vars: VarMap<'arn, 'grm>,
        arena: &'arn Arena<VarMapNode<'arn, 'grm>>
    ) -> ParseIndex {
        match value {
            ActionResult::Construct(span, constructor, args) => {
                let inner = match *constructor {
                    "Type" => {
                        assert_eq!(args.len(), 0);
                        SourceExpr::Type
                    }
                    "Let" => {
                        assert_eq!(args.len(), 3);
                        SourceExpr::Let(
                            args[0].get_value(program).to_string(),
                            self.insert_from_action_result(&args[1], program, vars, arena),
                            self.insert_from_action_result(&args[2], program, vars, arena),
                        )
                    }
                    "Variable" => {
                        assert_eq!(args.len(), 1);
                        let name = args[0].get_value(program).to_string();
                        if let Some(value) = vars.get(&name) {
                            SourceExpr::ScopeExit(self.insert_from_action_result(
                                value.as_value().expect("Parsed to value").as_ref(),
                                program,
                                vars, arena
                            ))
                        } else {
                            SourceExpr::Variable(name)
                        }
                    }
                    "FnType" => {
                        assert_eq!(args.len(), 3);
                        SourceExpr::FnType(
                            args[0].get_value(program).to_string(),
                            self.insert_from_action_result(&args[1], program, vars, arena),
                            self.insert_from_action_result(&args[2], program, vars, arena),
                        )
                    }
                    "FnConstruct" => {
                        assert_eq!(args.len(), 3);
                        SourceExpr::FnConstruct(
                            args[0].get_value(program).to_string(),
                            self.insert_from_action_result(&args[1], program, vars, arena),
                            self.insert_from_action_result(&args[2], program, vars, arena),
                        )
                    }
                    "FnDestruct" => {
                        assert_eq!(args.len(), 2);
                        SourceExpr::FnDestruct(
                            self.insert_from_action_result(&args[0], program, vars, arena),
                            self.insert_from_action_result(&args[1], program, vars, arena),
                        )
                    }
                    "ScopeDefine" => {
                        assert_eq!(args.len(), 2);
                        SourceExpr::ScopeDefine(
                            self.insert_from_action_result(&args[0], program, vars, arena),
                            Self::parse_guid(&args[1]),
                        )
                    }
                    "ScopeEnter" => {
                        assert_eq!(args.len(), 2);
                        SourceExpr::ScopeEnter(
                            self.insert_from_action_result(&args[0], program, vars, arena),
                            Self::parse_guid(&args[1]),
                        )
                    }
                    _ => unreachable!(),
                };
                self.store(inner, *span)
            }
            ActionResult::WithEnv(new_vars, ar) => {
                self.insert_from_action_result(ar, program, vars.extend(new_vars.iter_cloned(), arena), arena)
            }
            _ => unreachable!("Parsing an expression always returns a Construct"),
        }
    }

    fn parse_guid(ar: &ActionResult) -> Guid {
        let ActionResult::Guid(v) = ar else {
            unreachable!()
        };
        Guid(*v)
    }
}
