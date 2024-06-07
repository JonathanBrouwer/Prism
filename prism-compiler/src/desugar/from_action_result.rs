use prism_parser::rule_action::action_result::ActionResult;
use crate::desugar::{ParseEnv, SourceExpr, ParseIndex};

impl ParseEnv {
    pub fn insert_from_action_result<'grm>(
        &mut self,
        value: &ActionResult<'_, 'grm>,
        program: &str,
    ) -> ParseIndex {
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
                    self.insert_from_action_result(&args[1], program),
                    self.insert_from_action_result(&args[2], program),
                )
            }
            "Variable" => {
                assert_eq!(args.len(), 1);
                SourceExpr::Variable(
                    args[0].get_value(program).to_string(),
                ) 
            }
            "FnType" => {
                assert_eq!(args.len(), 3);
                SourceExpr::FnType(
                    args[0].get_value(program).to_string(),
                    self.insert_from_action_result(&args[1], program),
                    self.insert_from_action_result(&args[2], program),
                )
            }
            "FnConstruct" => {
                assert_eq!(args.len(), 3);
                SourceExpr::FnConstruct(
                    args[0].get_value(program).to_string(),
                    self.insert_from_action_result(&args[1], program),
                    self.insert_from_action_result(&args[2], program),
                )
            }
            "FnDestruct" => {
                assert_eq!(args.len(), 2);
                SourceExpr::FnDestruct(
                    self.insert_from_action_result(&args[0], program),
                    self.insert_from_action_result(&args[1], program),
                )
            }
            _ => unreachable!(),
        };
        self.store(inner, *span)
    }
}
