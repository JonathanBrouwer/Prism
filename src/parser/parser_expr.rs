use crate::parser::parser::JonlaParser;
use crate::parser::parser_expr_functiondefinition::FunctionDefinition;
use crate::parser::parser_expr_identifiereval::IdentifierEval;
use crate::parser::parser_expr_block::ExprBlock;

#[derive(Debug, Clone)]
pub enum Expression<'a> {
    FunctionDefinition(FunctionDefinition<'a>),
    IdentifierEval(IdentifierEval<'a>),
    ExprBlock(ExprBlock<'a>),
    ExprSequence(ExprSequence<'a>),
}

#[derive(Debug, Clone)]
pub struct ExprSequence<'a> {
    exprs: Vec<Expression<'a>>,
}

impl<'a> JonlaParser<'a> {
    pub fn parse_expr(&mut self) -> Result<Expression<'a>, String> {
        let exprs = self.one_or_more(|p|
            p.or(vec![
                |p| Self::parse_expr_block(p).map(Expression::ExprBlock),
                |p| Self::parse_expr_function(p).map(Expression::FunctionDefinition),
                |p| Self::parse_expr_identifiereval(p).map(Expression::IdentifierEval),
        ]))?;
        if exprs.len() == 1 {
            Ok(exprs.into_iter().next().unwrap())
        } else {
            Ok(Expression::ExprSequence(ExprSequence { exprs }))
        }
    }
}
