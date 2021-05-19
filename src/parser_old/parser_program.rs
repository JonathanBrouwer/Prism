use std::fmt;
use std::fmt::{Display, Formatter, Debug};

use crate::parser::parser::JonlaParser;
use crate::parser::parser_stmt::Statement;
use crate::lexer::lexer::LexerTokenType::Line;

pub struct Program<'a> {
    statements: Vec<Statement<'a>>,
}

impl<'a> JonlaParser<'a> {
    pub fn parse_program(&mut self) -> Result<Program<'a>, String> {
        let mut stmts = Vec::new();
        while self.peek().is_some() {
            let stmt = self.parse_stmt()?;
            if self.peek().is_some() {
                self.expect(Line)?;
            }
            while self.peek().is_some() && self.peek().unwrap().token == Line {
                self.next();
            }
            stmts.push(stmt);
        }
        return Ok(Program { statements: stmts });
    }
}

impl<'a> Debug for Program<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for st in &self.statements {
            writeln!(f, "{:?}", st)?;
        }
        Ok(())
    }
}
