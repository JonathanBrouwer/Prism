use crate::parser_core::adaptive::GrammarState;
use crate::parser_core::error::ParseError;
use crate::parser_core::parser::Parser;
use crate::parser_core::presult::PResult;
use crate::parser_core::primitives::end;
use crate::parser_core::stream::StringStream;
use crate::parser_sugar::error_printer::ErrorLabel;
use crate::parser_sugar::parser_rule::{parser_rule, PState, ParserContext};

pub fn parser_with_layout<'a, 'b: 'a, 'grm: 'b, O, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    rules: &'b GrammarState<'b, 'grm>,
    sub: &'a impl Parser<'b, 'grm, O, E, PState<'b, 'grm, E>>,
) -> impl Parser<'b, 'grm, O, E, PState<'b, 'grm, E>> + 'a {
    move |pos: StringStream<'grm>, cache: &mut PState<'b, 'grm, E>, context: &ParserContext<'b, 'grm>| -> PResult<'grm, O, E> {
        if context.layout_disabled || !rules.contains_rule("layout") {
            return sub.parse(pos, cache, context);
        }

        //Start attemping to parse layout
        let mut res = PResult::new_ok((), pos);
        loop {
            let (new_res, success) = res.merge_seq_opt_parser(sub, cache, context);
            if success {
                return new_res.map(|(_, o)| o.unwrap());
            }

            res = new_res
                .merge_seq_parser(
                    &parser_rule(
                        rules,
                        "layout"
                    ),
                    cache,
                    &ParserContext {
                        layout_disabled: true,
                        ..*context
                    },
                )
                .map(|_| ());
            if res.is_err() {
                return res.map(|_| unreachable!());
            }
        }
    }
}

pub fn full_input_layout<'a, 'b: 'a, 'grm: 'b, O, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    rules: &'b GrammarState<'b, 'grm>,
    sub: &'a impl Parser<'b, 'grm, O, E, PState<'b, 'grm, E>>,
) -> impl Parser<'b, 'grm, O, E, PState<'b, 'grm, E>> + 'a {
    move |stream: StringStream<'grm>, cache: &mut PState<'b, 'grm, E>, context: &ParserContext<'b, 'grm>| -> PResult<'grm, O, E> {
        let res = sub.parse(stream, cache, context);
        res.merge_seq_parser(&parser_with_layout(rules, &end()), cache, context)
            .map(|(o, _)| o)
    }
}
