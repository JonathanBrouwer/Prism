#[allow(unused)]
#[rustfmt::skip]
mod autogen;

use crate::autogen::ast::Term;
use jonla_macros::grammar;
use jonla_macros::grammar::GrammarFile;
use jonla_macros::parser::ActionResult;

// fn term_from_action_result<'grm, 'src: 'grm>(a: ActionResult<'grm>, input: &'src str) -> Term<'src> {
//     match a {
//         ActionResult::Construct(name, args) => {
//             match name {
//                 ""
//             }
//         }
//         _ => unreachable!(),
//     }
//     todo!()
// }

fn main() {
    let s = include_str!("../resources/grammar");
    let grammar: GrammarFile = match grammar::grammar_def::toplevel(&s) {
        Ok(ok) => ok,
        Err(err) => {
            panic!("{}", err);
        }
    };
    println!("");
}
