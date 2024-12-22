use crate::action::action_result::ActionResult;
use crate::core::cache::Allocs;
use crate::core::input::Input;
use crate::core::parsable::{Parsable, Parsed};
use crate::core::span::Span;
use crate::core::state::ParserState;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::rule_action::RuleAction;
use crate::parser::var_map::{VarMap, VarMapValue};

impl<'arn, 'grm, E: ParseError<L = ErrorLabel<'grm>>> ParserState<'arn, 'grm, E> {
    pub fn apply_action(
        &self,
        rule: &RuleAction<'arn, 'grm>,
        span: Span,
        vars: VarMap<'arn, 'grm>,
    ) -> Parsed<'arn, 'grm> {
        match rule {
            RuleAction::Name(name) => {
                if let Some(ar) = vars.get(name) {
                    if let VarMapValue::Value(v) = ar {
                        *v
                    } else {
                        panic!("")
                    }
                } else {
                    panic!("Name '{name}' not in context")
                }
            }
            RuleAction::InputLiteral(lit) => self.alloc.alloc(Input::Literal(*lit)).to_parsed(),
            RuleAction::Construct(namespace, name, args) => {
                let args_vals = self
                    .alloc
                    .alloc_extend(args.iter().map(|a| self.apply_action(a, span, vars)));
                (self
                    .parsables
                    .get(namespace)
                    .expect("Namespace exists")
                    .from_construct)(span, name, args_vals, self.alloc)
            }
            RuleAction::Value(ar) => self
                .alloc
                .alloc(ActionResult::WithEnv(vars, *ar))
                .to_parsed(),
        }
    }
}
