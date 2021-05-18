use crate::lexer::lexer::LexerTokenType::{BlockStart, BlockStop, Line, ParenClose, ParenOpen};
use crate::parser::parser::JonlaParser;
use crate::parser::parser_expr::Expression;

#[derive(Debug, Clone)]
pub struct ExprBlock<'a> {
    exprs: Vec<Expression<'a>>,
}

impl<'a> JonlaParser<'a> {
    pub fn parse_expr_block(&mut self) -> Result<ExprBlock<'a>, String> {
        self.expect(Line)?;
        self.expect(BlockStart)?;

        let mut exprs = Vec::new();
        loop {
            let cursor_old = self.cursor;
            if let Ok(v) = self.parse_expr() {
                self.expect(Line)?;
                exprs.push(v);
            } else {
                self.cursor = cursor_old;
                break;
            }
        }

        self.expect(BlockStop)?;

        Ok(ExprBlock{exprs})
    }
}
