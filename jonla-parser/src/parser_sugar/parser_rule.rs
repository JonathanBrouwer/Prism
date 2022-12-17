use crate::parser_core::adaptive::{BlockState, GrammarState};
use crate::parser_core::context::{Ignore, PCache, ParserContext, PR};
use crate::parser_core::error::error_printer::ErrorLabel;
use crate::parser_core::error::ParseError;
use crate::parser_core::parser::Parser;
use crate::parser_core::stream::StringStream;
use crate::parser_sugar::parser_rule_body::parser_body_cache_recurse;
use std::collections::HashMap;

pub fn parser_rule<'a, 'b: 'a, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    rules: &'b GrammarState<'b, 'grm>,
    rule: &'grm str,
) -> impl Parser<'b, 'grm, PR<'grm>, E> + 'a {
    move |stream: StringStream<'grm>,
          cache: &mut PCache<'b, 'grm, E>,
          context: &ParserContext<'b, 'grm>| {
        let body: &'b Vec<BlockState<'b, 'grm>> =
            rules.get(rule).expect(&format!("Rule not found: {rule}"));
        let mut res = parser_body_cache_recurse(rules, body).parse(
            stream,
            cache,
            &ParserContext {
                prec_climb_this: Ignore(None),
                prec_climb_next: Ignore(None),
                ..context.clone()
            },
        );
        res.add_label_implicit(ErrorLabel::Debug(stream.span_to(res.get_stream()), rule));
        res.map(|(_, v)| (HashMap::new(), v))
    }
}
