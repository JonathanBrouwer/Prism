use jonla_parser::error::error_printer::print_set_error;
use jonla_parser::grammar::GrammarFile;
use jonla_parser::parse_grammar;
use jonla_parser::META_GRAMMAR;

fn get_new_grammar(input: &str) -> GrammarFile {
    match parse_grammar(input) {
        Ok(o) => o,
        Err(es) => {
            for e in es {
                // print_tree_error(e, "file", input, true);
                print_set_error(e, input, true);
            }
            panic!();
        }
    }
}

#[test]
pub fn test_bootstrap() {
    let grammar: &'static GrammarFile = &META_GRAMMAR;

    let input = include_str!("../resources/meta.grammar");
    let grammar2 = get_new_grammar(input);

    assert_eq!(grammar, &grammar2); // Safety check
}
