use prism_compiler::parse_prism;

fn main() {
    let input = include_str!("../resources/program.pr");
    let Some((mut tc_env, root)) = parse_prism(input) else {
        return;
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
        Err(_) => {
            println!("Type check failed.");
            return;
        }
    }

    println!(
        "> Evaluated\n====================\n{}\n\n",
        tc_env.index_to_br_string(root)
    );
}
