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
                ValueOrigin::FreeSub(_) => todo!(),
                ValueOrigin::Failure => todo!(),
            }
        }
    }
}

#[derive(Diagnostic)]
#[diag(title = "Expected type", env = PrismDb)]
pub struct ExpectedType {
    #[sugg(label = format!("Expected a type, found value of type: {}", env.index_to_sm_string(self.index)))]
    pub(crate) index: CoreIndex,
}
