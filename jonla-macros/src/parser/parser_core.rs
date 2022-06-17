use crate::grammar::CharClass;
use crate::parser::parser_result::ParseResult;

pub struct ParserState<'src> {
    input: &'src str,
}

impl<'src> ParserState<'src> {
    pub fn new(input: &'src str) -> Self {
        ParserState { input }
    }
    pub fn parse_charclass(&mut self, pos: usize, cc: &CharClass) -> ParseResult<(usize, usize)> {
        match self.input[pos..].chars().next() {
            Some(c) if cc.contains(c) => {
                ParseResult::new_ok((pos, pos + c.len_utf8()), pos + c.len_utf8(), pos + c.len_utf8(), false)
            }
            _ => {
                ParseResult::new_err((pos, pos), pos, pos)
            }
        }
    }

    pub fn parse_sequence(&mut self, pos: usize, left: Box<dyn FnOnce(&mut ParserState<'src>, usize) -> ParseResult<(usize, usize)>>) -> ParseResult<(usize, usize)> {

    }
}