use crate::core_parser::input::Input;

/// Represents a parser that parsed its value successfully.
/// It parsed the value of type `O`.
/// It also stores the best error encountered during parsing, and the position AFTER the parsed value in `pos`.
#[derive(Clone)]
pub struct ParseResult<'src, O: Clone> {
    pub result: O,
    pub pos: Input<'src>,
    pub pos_err: Input<'src>,
    pub ok: bool,
    pub recovered: bool,
}

impl<'src, O: Clone> ParseResult<'src, O> {
    /// Maps the result of this ParseSuccess, using a mapping function.
    pub fn map<F, ON: Clone>(self, mapfn: F) -> ParseResult<'src, ON>
    where
        F: Fn(O) -> ON,
    {
        ParseResult {
            result: mapfn(self.result),
            pos: self.pos,
            pos_err: self.pos_err,
            ok: self.ok,
            recovered: self.recovered,
        }
    }

    pub fn new(
        result: O,
        pos: Input<'src>,
        pos_err: Input<'src>,
        ok: bool,
        recovered: bool,
    ) -> Self {
        ParseResult {
            result,
            pos,
            pos_err,
            ok,
            recovered,
        }
    }

    pub fn new_ok(result: O, pos: Input<'src>, pos_err: Input<'src>, recovered: bool) -> Self {
        ParseResult {
            result,
            pos,
            pos_err,
            ok: true,
            recovered,
        }
    }

    pub fn new_err(result: O, pos: Input<'src>, pos_err: Input<'src>) -> Self {
        ParseResult {
            result,
            pos,
            pos_err,
            ok: false,
            recovered: false,
        }
    }
}
