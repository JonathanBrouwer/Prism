use crate::lexer::lexer::LexerTokenType::{BlockStart, BlockStop, Line, ParenClose, ParenOpen};
use crate::parser::parser::JonlaParser;
use crate::parser::parser_expr::Expression;

#[derive(Debug, Clone)]
pub struct FunctionDefinition<'a> {
    name: &'a str,
    tpe: FunctionType<'a>,
    body: Box<Expression<'a>>,
}

impl<'a> JonlaParser<'a> {
    pub fn parse_expr_function(&mut self) -> Result<FunctionDefinition<'a>, String> {
        self.expect_keyword("fn")?;
        let name = self.expect_identifier()?;
        self.expect_keyword(":")?;
        let tpe = self.parse_expr_function_type()?;
        self.expect_keyword("=")?;
        let body = Box::new(self.parse_expr()?);

        Ok(FunctionDefinition { name, tpe, body })
    }
}

#[derive(Debug, Clone)]
pub struct FunctionType<'a> {
    inputs: Vec<(&'a str, Expression<'a>)>,
    output: Box<Expression<'a>>,
}

impl<'a> JonlaParser<'a> {
    pub fn parse_expr_function_type(&mut self) -> Result<FunctionType<'a>, String> {
        let mut inputs = Vec::new();
        loop {
            let cursor_old = self.cursor;

            let popen = self.expect(ParenOpen);
            if let Ok(_) = popen {
                let name = self.expect_identifier()?;
                self.expect_keyword(":")?;
                let tpe = self.parse_expr()?;
                self.expect(ParenClose)?;
                inputs.push((name, tpe));
            } else {
                self.cursor = cursor_old;
                break;
            }
        }

        self.expect_keyword("->")?;
        let output = Box::new(self.parse_expr()?);

        Ok(FunctionType { inputs, output })
    }
}
