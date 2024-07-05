use prism_compiler::desugar::{ParseEnv, ParseIndex};
use prism_compiler::lang::TcEnv;
use prism_compiler::parser::{GRAMMAR, parse_prism};
use prism_parser::error::aggregate_error::AggregatedParseError;
use prism_parser::error::set_error::SetError;
use prism_parser::parser::parser_instance::run_parser_rule;
use prism_parser::parser::var_map::VarMap;

fn main() {
    let input = include_str!("../resources/program.pr");


    let mut penv = ParseEnv::default();
    let idx = match run_parser_rule::<SetError, _>(&GRAMMAR, "expr", input, |r, allocs| {
        println!("> Action result\n====================\n{}\n\n", r.to_string(input));
        penv.insert_from_action_result(r, input, VarMap::default(), &allocs.alo_varmap)
    }) {
        Ok(idx) => idx,
        Err(e) => {
            e.eprint().unwrap();
            return;
        }
    };

    println!(
        "> Parsed program\n====================\n{}\n\n",
        penv.index_to_string(idx)
    );

    let mut tc_env = TcEnv::default();
    let root = tc_env.insert_parse_env(&penv, idx);

    println!(
        "> Desugared program\n====================\n{}\n\n",
        tc_env.index_to_string(root),
    );

    match tc_env.type_check(root) {
        Ok(i) => println!(
            "> Type of program\n====================\n{}\n\n",
            tc_env.index_to_br_string(i)
        ),
        Err(e) => {
            e.eprint(&mut tc_env, input).unwrap();
            return;
        }
    }

    println!(
        "> Evaluated\n====================\n{}\n\n",
        tc_env.index_to_br_string(root)
    );
}
