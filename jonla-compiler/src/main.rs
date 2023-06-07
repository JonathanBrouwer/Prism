pub mod coc;

use crate::coc::env::Env;
use crate::coc::Expr;
use coc::type_check::tc;
use jonla_parser::error::error_printer::print_set_error;
use jonla_parser::grammar::grammar::GrammarFile;
use jonla_parser::grammar::run::run_parser_rule;
use jonla_parser::parse_grammar;

fn main() {
    let grammar = include_str!("../resources/grammar");
    let grammar: GrammarFile = match parse_grammar(grammar) {
        Ok(ok) => ok,
        Err(es) => {
            for e in es {
                print_set_error(e, grammar, false);
            }
            return;
        }
    };

    let input = include_str!("../resources/program.jnl");
    let r: Result<_, _> = run_parser_rule(&grammar, "block", input);
    let r = match r {
        Ok(o) => o,
        Err(es) => {
            for e in es {
                print_set_error(e, input, false)
            }
            return;
        }
    };
    let expr = Expr::from_action_result(&r.1, input);
    println!("Program:\n{}", &expr);

    let typ = match tc(&expr, &Env::new()) {
        Ok(typ) => typ,
        Err(err) => {
            println!("Type error:\n{err:?}");
            return;
        }
    };
    println!("Type:\n{typ}");
}
