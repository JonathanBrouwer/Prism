use crate::parser_core::error::ParseError;
use crate::parser_core::parser::Parser;
use crate::parser_core::stream::StringStream;
use crate::parser_sugar::error_printer::ErrorLabel;
use crate::parser_sugar::parser_rule::{PState};

pub fn parse_with_recovery<'a, 'b: 'a, 'grm: 'b, O, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    sub: &'a impl Parser<'grm, O, E, PState<'b, 'grm, E>>,
    stream: StringStream<'grm>,
    state: &mut PState<'b, 'grm, E>,
) -> Result<O, Vec<E>> {
    match sub.parse(stream, state).collapse() {
        Ok(o) => Ok(o),
        Err(e) => Err(vec![e])
    }
}
