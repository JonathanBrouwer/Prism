use crate::core::input::Input;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::span::Span;
use crate::core::state::ParserState;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::escaped_string::EscapedString;
use crate::grammar::rule_action::RuleAction;
use crate::parsable::env_capture::EnvCapture;
use crate::parsable::parsed::Parsed;
use crate::parsable::ParseResult;
use crate::parser::var_map::VarMap;

impl<'arn, 'grm: 'arn, Env, E: ParseError<L = ErrorLabel<'grm>>> ParserState<'arn, 'grm, Env, E> {
    pub fn apply_action(
        &self,
        rule: &RuleAction<'arn, 'grm>,
        span: Span,
        vars: VarMap<'arn, 'grm>,
        penv: &mut Env,
    ) -> Parsed<'arn, 'grm> {
        match rule {
            RuleAction::Name(name) => {
                if let Some(ar) = vars.get(name) {
                    *ar
                } else {
                    panic!("Name '{name}' not in context")
                }
            }
            RuleAction::InputLiteral(lit) => self.alloc.alloc(Input::Literal(*lit)).to_parsed(),
            RuleAction::Construct(namespace, name, args) => {
                let ns = self
                    .parsables
                    .get(namespace)
                    .unwrap_or_else(|| panic!("Namespace '{namespace}' exists"));

                let mut builder = (ns.build)(name, self.alloc, self.input, penv);
                for (i, arg) in args.iter().enumerate() {
                    let arg = self.apply_action(arg, span, vars, penv);
                    (ns.arg)(&mut builder, i, arg, self.alloc, self.input, penv);
                }
                (ns.finish)(&mut builder, span, self.alloc, self.input, penv)
            }
            RuleAction::Value(parsed) => self
                .alloc
                .alloc(EnvCapture {
                    env: vars,
                    value: *parsed,
                })
                .to_parsed(),
        }
    }
}
