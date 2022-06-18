use crate::grammar::{CharClass, RuleBody};
use crate::parser::parser_result::ParseResult;
use std::collections::HashMap;
use std::marker::PhantomData;

pub struct ParserState<'grm, 'src> {
    input: &'src str,
    _ph: PhantomData<&'grm str>,
}

impl<'grm, 'src> ParserState<'grm, 'src> {
    pub fn new(input: &'src str) -> Self {
        ParserState {
            input,
            _ph: PhantomData::default(),
        }
    }
    pub fn parse_charclass(&mut self, pos: usize, cc: &CharClass) -> ParseResult<(usize, usize)> {
        match self.input[pos..].chars().next() {
            Some(c) if cc.contains(c) => {
                ParseResult::new_ok((pos, pos + c.len_utf8()), pos + c.len_utf8())
            }
            _ => ParseResult::new_err((pos, pos), pos),
        }
    }

    ///
    /// Recovery:
    /// - If left matched zero input, don't bother continuing, we'll succeed higher up
    /// - If left matched some amount, try to continue and match more
    pub fn parse_sequence(
        &mut self,
        pos: usize,
        left: Box<dyn Fn(&mut ParserState<'grm, 'src>, usize) -> ParseResult<(usize, usize)>>,
        right: Box<dyn Fn(&mut ParserState<'grm, 'src>, usize) -> ParseResult<(usize, usize)>>,
    ) -> ParseResult<(usize, usize)> {
        let res_left: ParseResult<(usize, usize)> = left(self, pos);
        if res_left.ok {
            let res_right: ParseResult<(usize, usize)> = right(self, res_left.pos);
            if res_right.ok {
                ParseResult::new_ok((res_left.result.0, res_right.result.1), res_right.pos)
            } else {
                ParseResult::new_err((res_left.result.0, res_right.result.1), res_right.pos)
            }
        } else {
            ParseResult::new_err((res_left.result.0, res_left.result.1), res_left.pos)
        }
    }

    // pub fn parse_choice(
    //     &mut self,
    //     pos: usize,
    //     left: Box<dyn Fn(&mut ParserState<'src>, usize) -> ParseResult<(usize, usize)>>,
    //     right: Box<dyn Fn(&mut ParserState<'src>, usize) -> ParseResult<(usize, usize)>>,
    // ) -> ParseResult<(usize, usize)> {
    //
    // }

    // pub fn parse_cache_recurse(
    //     &mut self,
    //     pos: usize,
    //     sub: Box<dyn Fn(&mut ParserState<'src>, usize) -> ParseResult<(usize, usize)>>,
    //     id: &str?,
    // ) -> ParseResult<(usize, usize)> {
    //
    // }
}
