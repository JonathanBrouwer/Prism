use crate::parser_core::error::ParseError;
use crate::parser_core::parser::Parser;
use crate::parser_core::stream::StringStream;
use crate::parser_sugar::error_printer::ErrorLabel;
use crate::parser_sugar::parser_rule::{ParserContext, PState};

pub fn parse_with_recovery<'a, 'b: 'a, 'grm: 'b, O, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    sub: &'a impl Parser<'b, 'grm, O, E, PState<'b, 'grm, E>>,
    stream: StringStream<'grm>,
    cache: &mut PState<'b, 'grm, E>,
    context: &ParserContext<'b, 'grm>,
) -> Result<O, Vec<E>> {
    match sub.parse(stream, cache, context).collapse() {
        Ok(o) => Ok(o),
        Err(e) => Err(vec![e])
    }
}
