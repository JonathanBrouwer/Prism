use prism_compiler::lang::{TcEnv, UnionIndex};
use prism_compiler::parser::parse_prism;
use prism_parser::error::aggregate_errors::{AggregatedParseError, ResultExt};
use prism_parser::error::set_error::SetError;

fn main() {
    let input = include_str!("../resources/program.pr");

    let (mut tc_env, root) = match parse_prism(input) {
        Ok(v) => v,
        Err(e) => {
            e.eprint().unwrap();
            return;
        }
    };

    println!(
        "> Program\n====================\n{}\n\n",
        tc_env.index_to_sm_string(root),
    );

    match tc_env.type_check(root) {
        Ok(i) => println!(
            "> Type of program\n====================\n{}\n\n",
            tc_env.index_to_br_string(i)
        ),
        Err(e) => {
            for e in e {
                
            }
            println!("Type check failed.");
            return;
        }
    }

    println!(
        "> Evaluated\n====================\n{}\n\n",
        tc_env.index_to_br_string(root)
    );
}
