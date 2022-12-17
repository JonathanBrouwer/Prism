use crate::grammar::GrammarFile;
use crate::parser_core::adaptive::GrammarState;
use crate::parser_core::cache::ParserCache;
use crate::parser_core::context::{ParserContext, PR};
use crate::parser_core::error::error_printer::ErrorLabel;
use crate::parser_core::error::ParseError;
use crate::parser_core::recovery::parse_with_recovery;
use crate::parser_core::stream::StringStream;
use crate::parser_sugar::parser_layout::full_input_layout;
use crate::parser_sugar::parser_rule;

pub fn run_parser_rule<'a, 'b: 'a, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    rules: &'grm GrammarFile,
    rule: &'grm str,
    stream: StringStream<'grm>,
) -> Result<PR<'grm>, Vec<E>> {
    let context = ParserContext::new();
    let mut cache = ParserCache::new();
    let grammar_state = GrammarState::new(&rules);

    let x = parse_with_recovery(
        &full_input_layout(
            &grammar_state,
            &parser_rule::parser_rule(&grammar_state, rule),
        ),
        stream,
        &mut cache,
        &context,
    );
    x
}
