use bumpalo::Bump;
use prism_parser::core::cache::Allocs;
use prism_parser::error::aggregate_error::ParseResultExt;
use prism_parser::error::set_error::SetError;
use prism_parser::grammar::GrammarFile;
use prism_parser::parse_grammar;
use prism_parser::rule_action::RuleAction;
use prism_parser::META_GRAMMAR;

#[cfg_attr(not(debug_assertions), test)]
pub fn test_bootstrap() {
    let grammar: &'static GrammarFile<RuleAction> = &META_GRAMMAR;

    let input = include_str!("../resources/meta.grammar");
    let bump = Bump::new();
    let alloc = Allocs::new(&bump);
    let grammar2 = parse_grammar::<SetError>(input, alloc).unwrap_or_eprint();

    assert!(grammar == &grammar2, "Meta grammar is not up-to-date"); // Check if grammar file needs to be updated
}
