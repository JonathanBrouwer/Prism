use crate::parser::parser::JonlaParser;

#[derive(Debug, Clone)]
pub struct IdentifierEval<'a> {
    name: &'a str,
}

impl<'a> JonlaParser<'a> {
    pub fn parse_expr_identifiereval(&mut self) -> Result<IdentifierEval<'a>, String> {
        let name = self.expect_identifier()?;

        Ok(IdentifierEval { name })
    }
}
