use prism_compiler::parse_prism;

fn main() {
    let input = include_str!("../resources/program.pr");
    let Some(mut tc_env) = parse_prism(input) else {
        return
    };

    println!("> Program:\n==========\n{}\n==========", tc_env.index_to_string(tc_env.root, false).unwrap());

    match tc_env.type_check() {
        Ok(i) => println!("> Type:\n==========\n{}\n==========", tc_env.index_to_string(i, true).unwrap()),
        Err(_) => println!("Type check failed."),
    }
}

