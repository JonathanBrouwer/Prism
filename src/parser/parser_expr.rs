use crate::parser::parser::JonlaParser;
use crate::parser::parser_expr_functiondefinition::FunctionDefinition;
use crate::parser::parser_expr_identifiereval::IdentifierEval;

#[derive(Debug)]
pub enum Expression<'a> {
    FunctionDefinition(FunctionDefinition<'a>),
    IdentifierEval(IdentifierEval<'a>)
}

impl<'a> JonlaParser<'a> {
    pub fn parse_expr(&mut self) -> Result<Expression<'a>, String> {
        self.or(vec![
            |p| Self::parse_expr_function(p).map(Expression::FunctionDefinition),
            |p| Self::parse_expr_identifiereval(p).map(Expression::IdentifierEval),
        ])
    }
}


