use crate::lang::env::DbEnv;
use crate::lang::{CoreIndex, PrismDb, ValueOrigin};
use prism_diag::sugg::SuggestionArgument;
use prism_diag_derive::Diagnostic;
use prism_input::span::Span;

impl SuggestionArgument<PrismDb> for CoreIndex {
    fn span(&self, env: &PrismDb) -> Span {
        let mut origin = env.checked_origins[self.0];
        loop {
            match origin {
                ValueOrigin::SourceCode(span) => return span,
                ValueOrigin::TypeOf(i) => origin = env.checked_origins[i.0],
                ValueOrigin::FreeSub(s) => origin = env.checked_origins[s.0],
                ValueOrigin::Failure => todo!(),
            }
        }
    }
}

#[derive(Diagnostic)]
#[diag(title = "Expected type", env = PrismDb)]
pub struct ExpectedType {
    #[sugg(label = format!("Expected a type, found value of type: {}", env.index_to_sm_string(self.index)))]
    pub index: CoreIndex,
}

#[derive(Diagnostic)]
#[diag(title = "Expected function", env = PrismDb)]
pub struct ExpectedFn {
    #[sugg(label = format!("Expected a function, found value of type: {}", env.index_to_sm_string(self.index)))]
    pub index: CoreIndex,
}

#[derive(Diagnostic)]
#[diag(title = "Argument type mismatch in function application", env = PrismDb)]
pub struct ExpectedFnArg {
    #[sugg(label = format!("Found an argument of type: {}", env.index_to_sm_string(self.arg_type)))]
    pub arg_type: CoreIndex,
    #[sugg(label = format!("Function expects an argument of type: {}", env.index_to_sm_string(self.function_arg_type)))]
    pub function_type: CoreIndex,
    pub function_arg_type: CoreIndex,
}
