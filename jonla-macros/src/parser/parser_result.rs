/// Represents a parser that parsed its value successfully.
/// It parsed the value of type `O`.
/// It also stores the best error encountered during parsing, and the position AFTER the parsed value in `pos`.
#[derive(Clone)]
pub struct ParseResult<O: Clone> {
    pub result: O,
    pub pos: usize,
    pub ok: bool,
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
            ok: self.ok,
        }
    }

    pub fn new_ok(result: O, pos: usize) -> Self {
        ParseResult {
            result,
            pos,
            ok: true,
        }
    }

    pub fn new_err(result: O, pos: usize) -> Self {
        ParseResult {
            result,
            pos,
            ok: false,
        }
    }
}
