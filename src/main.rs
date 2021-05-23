use crate::lexer::lexer::*;
use crate::parser::customizable_parser::*;

mod lexer;
mod parser;

fn main() {
    let source = include_str!("../resources/test.jnl");

    // Lexer
    let lexer = ActualLexer::new(source);
    let lexer_res: Vec<LexerItem> = lexer.collect();

    let print_lexer_result = true;
    if print_lexer_result {
        println!("------------------");
        println!("Lexer tokens:");

        lexer_res.iter().for_each(|l| {
            println!("[{:?}] {:?}", l.span, l.token);
        });
    }

    lexer_res.as_slice();

    // Parse file
    let parser = CustomizableParser::from_vec(vec![
        vec![ParseGroup{ rules: vec![
            ParseRule::Seq(vec![ParseRule::SubLevelExpr, ParseRule::Expect(LexerToken::EOF)] ),
        ] }],
        vec![ParseGroup{ rules: vec![
            ParseRule::Seq(vec![ParseRule::SubLevelExpr, ParseRule::Expect(LexerToken::Name("+")), ParseRule::SameLevelExpr] ),
            ParseRule::Seq(vec![ParseRule::SubLevelExpr] ),
        ] }],
        // vec![ParseGroup{ rules: vec![
        //     ParseRule{ parts: vec![ParseRulePart::SubLevelExpr, ParseRulePart::Expect(LexerToken::Name("*")), ParseRulePart::SameLevelExpr] },
        //     ParseRule{ parts: vec![ParseRulePart::SubLevelExpr] },
        // ] }],
        vec![ParseGroup{ rules: vec![
            ParseRule::Seq(vec![ParseRule::Bind(LexerTokenType::Name)] ),
        ] }],
    ]);
    let program = match parser.parse(lexer_res.as_slice()) {
        Ok(program) => program,
        Err(e) => {
            println!("Parse error: {}", e);
            return
        }
    };
    println!("Success");
    // program.print();
}
