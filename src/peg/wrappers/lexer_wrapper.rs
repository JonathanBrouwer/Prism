use std::fmt::{Debug, Display};
use std::rc::Rc;
use crate::{ParseError, Parser, ParseSuccess};
use crate::peg::input::Input;

#[derive(Clone)]
pub struct LexerWrapper<I: Input<InputElement=char>, O: Debug + Display + PartialEq + Eq + Clone + Copy> {
    wrapped: I,
    lexer: Rc<dyn Parser<I , O>>
}

impl<I: Input<InputElement=char>, O: Debug + Display + PartialEq + Eq + Clone + Copy> Input for LexerWrapper<I, O> {
    type InputElement = O;

    fn next(&self) -> Result<ParseSuccess<LexerWrapper<I, O>, O>, ParseError<LexerWrapper<I, O>>> {
        let map_pos = |pos| LexerWrapper { wrapped: pos, lexer: self.lexer.clone() };
        let map_err = |err: ParseError<_>| ParseError { errors: err.errors, pos: map_pos(err.pos) };
        let map_suc = |suc: ParseSuccess<_, _>| ParseSuccess { result: suc.result, pos: map_pos(suc.pos), best_error: suc.best_error.map(map_err) };
        self.lexer.parse(self.wrapped.clone()).map(map_suc).map_err(map_err)
    }

    fn pos(&self) -> usize {
        self.wrapped.pos()
    }

    fn src_str<'a>(&'a self) -> Box<dyn ToString + 'a> {
        self.wrapped.src_str()
    }

    fn src_slice(&self) -> (usize, usize) {
        self.wrapped.src_slice()
    }
}