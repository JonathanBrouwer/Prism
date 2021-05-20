use crate::lexer::lexer::*;
use crate::parser::parser_program::*;

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
    let program = match parse_program_file(lexer_res.as_slice()) {
        Ok(program) => program,
        Err(e) => {
            println!("Parse error: {:?}", e);
            return
        }
    };
    program.print();




    // let mut parser_old = JonlaParser::new(source, lexer_tokens);
    // let program = parser_old.parse_program();
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
