use prism_input::span::Span;

pub trait SuggestionArgument<Env> {
    fn span(&self, env: &Env) -> Span;
}

impl<Env> SuggestionArgument<Env> for Span {
    fn span(&self, _env: &Env) -> Span {
        *self
    }
}
