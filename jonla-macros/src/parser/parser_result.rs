/// Represents a parser that parsed its value successfully.
/// It parsed the value of type `O`.
/// It also stores the best error encountered during parsing, and the position AFTER the parsed value in `pos`.
#[derive(Clone)]
pub struct ParseResult<O: Clone> {
    pub result: Option<O>,
    pub pos: usize,
}

impl<O: Clone> ParseResult<O> {
    pub fn map<F, ON: Clone>(self, mapfn: F) -> ParseResult<ON>
    where
        F: FnOnce(O) -> ON,
    {
        ParseResult {
            result: self.result.map(mapfn),
            pos: self.pos,
        }
    }

    pub fn new_ok(result: O, pos: usize) -> Self {
        ParseResult {
            result: Some(result),
            pos,
        }
    }

    pub fn new_err(pos: usize) -> Self {
        ParseResult { result: None, pos }
    }

    pub fn is_ok(&self) -> bool {
        self.result.is_some()
    }
}
