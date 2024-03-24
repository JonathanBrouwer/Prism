use prism_compiler::parse_prism;
use prism_compiler::coc::TcEnv;

fn main() {
    let input = include_str!("../resources/program.pr");
    let Some(mut tc_env) = parse_prism(input) else {
        return
    };

    println!("Program:\n{}", tc_env.to_string(tc_env.root).unwrap());

    match tc_env.type_check() {
        Ok(()) => println!("Type check ok."),
        Err(_) => println!("Type check failed."),
    }

    // let typ = match tc(&expr, &Env::new()) {
    //     Ok(typ) => typ,
    //     Err(err) => {
    //         println!("Type error:\n{err:?}");
    //         return;
    //     }
    // };
    // println!("Type:\n{typ}");
}

