/// Represents a parser that parsed its value successfully.
/// It parsed the value of type `O`.
/// It also stores the best error encountered during parsing, and the position AFTER the parsed value in `pos`.
#[derive(Clone)]
pub struct ParseResult<O: Clone> {
    pub result: O,
    pub pos: usize,
    pub pos_err: usize,
    pub ok: bool,
    pub recovered: bool,
}

impl<O: Clone> ParseResult<O> {
    /// Maps the result of this ParseSuccess, using a mapping function.
    pub fn map<F, ON: Clone>(self, mapfn: F) -> ParseResult<ON>
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
        pos: usize,
        pos_err: usize,
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

    pub fn new_ok(result: O, pos: usize, pos_err: usize, recovered: bool) -> Self {
        ParseResult {
            result,
            pos,
            pos_err,
            ok: true,
            recovered,
        }
    }

    pub fn new_err(result: O, pos: usize, pos_err: usize) -> Self {
        ParseResult {
            result,
            pos,
            pos_err,
            ok: false,
            recovered: false,
        }
    }
}
