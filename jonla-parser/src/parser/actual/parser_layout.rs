use crate::grammar::GrammarFile;
use crate::parser::actual::error_printer::ErrorLabel;
use crate::parser::actual::parser_rule::{parser_rule, ParserContext, PR};
use crate::parser::core::error::ParseError;
use crate::parser::core::parser::Parser;
use crate::parser::core::parser_state::ParserState;
use crate::parser::core::presult::PResult;
use crate::parser::core::primitives::end;
use crate::parser::core::stream::Stream;

pub fn parser_with_layout<
    'a,
    'b: 'a,
    'grm: 'b,
    O,
    S: Stream,
    E: ParseError<L = ErrorLabel<'grm>> + Clone,
>(
    rules: &'grm GrammarFile,
    sub: &'a impl Parser<O, S, E, ParserState<'b, PResult<PR<'grm>, E, S>>>,
    context: &'a ParserContext<'grm>,
) -> impl Parser<O, S, E, ParserState<'b, PResult<PR<'grm>, E, S>>> + 'a {
    move |pos: S, state: &mut ParserState<'b, PResult<PR<'grm>, E, S>>| -> PResult<O, E, S> {
        if context.layout_disabled || !rules.rules.contains_key("layout") {
            return sub.parse(pos, state);
        }

        //Start attemping to parse layout
        let mut res = PResult::new_ok((), pos);
        loop {
            let (new_res, success) = res.merge_seq_opt_parser(sub, state);
            if success {
                return new_res.map(|(_, o)| o.unwrap());
            }

            res = new_res
                .merge_seq_parser(
                    &parser_rule(
                        rules,
                        "layout",
                        &ParserContext {
                            layout_disabled: true,
                            ..*context
                        },
                    ),
                    state,
                )
                .map(|_| ());
            if res.is_err() {
                return res.map(|_| unreachable!());
            }
        }
    }
}

pub fn full_input_layout<
    'a,
    'b: 'a,
    'grm: 'b,
    O,
    S: Stream,
    E: ParseError<L = ErrorLabel<'grm>> + Clone,
>(
    rules: &'grm GrammarFile,
    sub: &'a impl Parser<O, S, E, ParserState<'b, PResult<PR<'grm>, E, S>>>,
    context: &'a ParserContext<'grm>,
) -> impl Parser<O, S, E, ParserState<'b, PResult<PR<'grm>, E, S>>> + 'a {
    move |stream: S, state: &mut ParserState<'b, PResult<PR<'grm>, E, S>>| -> PResult<O, E, S> {
        let res = sub.parse(stream, state);
        res.merge_seq_parser(&parser_with_layout(rules, &end(), context), state)
            .map(|(o, _)| o)
    }
}
