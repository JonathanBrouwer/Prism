use crate::parser::parser_base::JonlaParser;
use crate::lexer::lexer::LexerTokenType;

#[derive(Debug)]
pub struct Program<'a> {
    statements: Vec<Statement<'a>>,
}

impl<'a> JonlaParser<'a> {
    pub fn parse_program(&mut self) -> Result<Program<'a>, String> {
        let mut stmts = Vec::new();
        while self.peek().is_some() {
            let stmt = self.parse_statement()?;
            stmts.push(stmt);
        }
        return Ok(Program { statements: stmts });
    }
}

#[derive(Debug)]
pub enum Statement<'a> {
    DataDefinition(DataDefinition<'a>)
}

impl<'a> JonlaParser<'a> {
    pub fn parse_statement(&mut self) -> Result<Statement<'a>, String> {
        Ok(Statement::DataDefinition(self.parse_data()?))
    }
}

#[derive(Debug)]
pub struct DataDefinition<'a> {
    name: &'a str,
    constructors: Vec<DataDefinitionConstructor<'a>>
}

#[derive(Debug)]
pub struct DataDefinitionConstructor<'a> {
    name: &'a str
}

impl<'a> JonlaParser<'a> {
    pub fn parse_data(&mut self) -> Result<DataDefinition<'a>, String> {
        self.expect_keyword("data")?;
        let name = self.expect_identifier()?;
        self.expect_keyword("=")?;
        self.expect(LexerTokenType::Line)?;
        self.expect(LexerTokenType::BlockStart)?;

        let mut constructors = Vec::new();
        loop {
            let cursor_old = self.cursor;
            let cons = self.parse_data_constructor();
            if let Ok(cons) = cons {
                constructors.push(cons);
            } else {
                self.cursor = cursor_old;
                break;
            }
        }

        self.expect(LexerTokenType::BlockStop)?;

        Ok(DataDefinition{name, constructors})
    }

    pub fn parse_data_constructor(&mut self) -> Result<DataDefinitionConstructor<'a>, String> {
        let name = self.expect_identifier()?;
        self.expect(LexerTokenType::Line)?;
        Ok(DataDefinitionConstructor{name})
    }
}
