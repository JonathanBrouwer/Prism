use crate::lexer::lexer::*;
use logos::{Source};

mod lexer;

fn main() {
    let source = include_str!("../resources/test_block.jnl");

    // Stage 1
    let lexer = FinalLexer::new(source);
    let lines: Vec<LexerLine> = lexer.collect();

    let print_lexer_result = true;
    if print_lexer_result {
        println!("------------------");
        println!("Lexer tokens:");

        lines.iter().for_each(|l| {
            print!("{} - ", l.indent);
            for token in &l.tokens {
                print!("{} ", source.slice(token.span.clone()).unwrap().escape_debug());
            }
            println!();
        });
    }

    // let mut parser = JonlaParser::new(source, lexer_tokens);
    // let program = parser.parse_program();
    // let program = match program {
    //     Ok(program) => program,
    //     Err(err) => {
    //         println!("Parse errors:\n{}", err);
    //         return;
    //     }
    // };
    //
    // let print_parser_result = true;
    // if print_parser_result {
    //     println!("------------------");
    //     println!("Parser output:");
    //
    //     println!("{:?}", program)
    // }
}
