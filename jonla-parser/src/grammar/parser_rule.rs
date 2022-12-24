use crate::core::adaptive::{BlockState, GrammarState};
use crate::core::cache::PCache;
use crate::core::context::{Ignore, ParserContext, PR};
use crate::core::parser::Parser;
use crate::core::pos::Pos;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::parser_rule_body::parser_body_cache_recurse;
use std::collections::HashMap;

pub fn parser_rule<'a, 'b: 'a, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    rules: &'b GrammarState<'b, 'grm>,
    rule: &'grm str,
) -> impl Parser<'b, 'grm, PR<'grm>, E> + 'a {
    move |stream: Pos, cache: &mut PCache<'b, 'grm, E>, context: &ParserContext<'b, 'grm>| {
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
        res.add_label_implicit(ErrorLabel::Debug(stream.span_to(res.get_pos()), rule));
        res.map(|(_, v)| (HashMap::new(), v))
    }
}
