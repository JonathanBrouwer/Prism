use prism_parser::META_GRAMMAR;
use prism_parser::error::set_error::SetError;
use prism_parser::grammar::grammar_file::GrammarFile;
use prism_parser::parse_grammar;

#[test]
pub fn test_bootstrap() {
    let grammar: &'static GrammarFile = &META_GRAMMAR;

    let input = include_str!("../../prism_parser/resources/meta.pg");
    let (table, grammar2, _, errs) = parse_grammar::<SetError>(input);
    errs.unwrap_or_eprint(&table);

    assert_eq!(
        rmp_serde::to_vec_named(&grammar).unwrap(),
        rmp_serde::to_vec_named(&grammar2).unwrap(),
        "Meta grammar is not up-to-date"
    ); // Check if grammar file needs to be updated
}
