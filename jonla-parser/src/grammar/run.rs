use crate::core::adaptive::GrammarState;
use crate::core::cache::{Allocs, ParserCache};
use crate::core::context::{ParserContext, PR};
use crate::core::pos::Pos;
use crate::core::recovery::parse_with_recovery;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::grammar::GrammarFile;
use crate::grammar::parser_layout::full_input_layout;
use crate::grammar::parser_rule;

pub fn run_parser_rule<'a, 'b: 'a, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    rules: &'grm GrammarFile,
    rule: &'grm str,
    input: &'grm str,
    //TODO add args
) -> Result<PR<'grm>, Vec<E>> {
    let context = ParserContext::new();
    let bump = Allocs::new();
    let mut cache = ParserCache::new(input, &bump);

    let grammar_state = GrammarState::new(&rules);

    let x = parse_with_recovery(
        &full_input_layout(
            &grammar_state,
            &parser_rule::parser_rule(&grammar_state, rule, &vec![]),
        ),
        Pos::start(),
        &mut cache,
        &context,
    );
    x
}
