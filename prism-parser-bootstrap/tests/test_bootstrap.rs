use bumpalo::Bump;
use prism_parser::META_GRAMMAR;
use prism_parser::core::cache::Allocs;
use prism_parser::error::aggregate_error::ParseResultExt;
use prism_parser::error::set_error::SetError;
use prism_parser::grammar::grammar_file::GrammarFile;
use prism_parser::parse_grammar;

#[test]
pub fn test_bootstrap() {
    let grammar: &'static GrammarFile = &META_GRAMMAR;

    let input = include_str!("../resources/meta.pg");
    let bump = Bump::new();
    let alloc = Allocs::new(&bump);
    let grammar2 = parse_grammar::<SetError>(input, alloc).unwrap_or_eprint();

    assert_eq!(
        rmp_serde::to_vec_named(&grammar).unwrap(),
        rmp_serde::to_vec_named(&grammar2).unwrap(),
        "Meta grammar is not up-to-date"
    ); // Check if grammar file needs to be updated
}
