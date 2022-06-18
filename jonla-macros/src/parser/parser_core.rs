use crate::grammar::CharClass;
use crate::parser::parser_result::ParseResult;
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
    pub fn parse_charclass(&mut self, pos: usize, cc: &CharClass) -> ParseResult<()> {
        match self.input[pos..].chars().next() {
            Some(c) if cc.contains(c) => ParseResult::new_ok((), pos + c.len_utf8()),
            _ => ParseResult::new_err(pos),
        }
    }

    ///
    /// Recovery:
    /// - If left matched zero input, don't bother continuing, we'll succeed higher up
    /// - If left matched some amount, try to continue and match more
    pub fn parse_sequence<S: Clone, T: Clone>(
        &mut self,
        res_left: ParseResult<S>,
        right: impl Fn(&mut ParserState<'grm, 'src>, usize) -> ParseResult<T>,
    ) -> ParseResult<(S, T)> {
        match res_left.result {
            Some(s) => {
                let res_right: ParseResult<T> = right(self, res_left.pos);
                if res_right.is_ok() {
                    ParseResult::new_ok((s, res_right.result.unwrap()), res_right.pos)
                } else {
                    ParseResult::new_err(res_right.pos)
                }
            }
            None => res_left.map(|_| unreachable!()),
        }
    }

    pub fn parse_choice<T: Clone>(
        &mut self,
        pos: usize,
        res_left: ParseResult<T>,
        right: impl Fn(&mut ParserState<'grm, 'src>, usize) -> ParseResult<T>,
    ) -> ParseResult<T> {
        match res_left.result {
            Some(_) => res_left,
            None => {
                let mut res_right: ParseResult<T> = right(self, pos);
                if res_right.is_ok() {
                    res_right
                } else {
                    res_right.pos = res_left.pos.max(res_right.pos);
                    res_right
                }
            }
        }
    }

    // pub fn parse_cache_recurse(
    //     &mut self,
    //     pos: usize,
    //     sub: Box<dyn Fn(&mut ParserState<'src>, usize) -> ParseResult<(usize, usize)>>,
    //     id: &str?,
    // ) -> ParseResult<(usize, usize)> {
    //
    // }
}
