use std::hash::Hash;
use crate::lambday::lambday::LambdayTerm;

enum JonlaTerm<Sym: Eq + Hash + Clone> {
    LambdayTerm(LambdayTerm<Sym>)
}

struct JonlaLambdayParser {

}

// impl<'a, I: Input<InputElement=char>> Parser<I, LambdayTerm<&'a str>> for JonlaLambdayParser {
//     fn parse(&self, input: I) -> Result<ParseSuccess<I, LambdayTerm<&'a str>>, ParseError<I>> {
//         Choice { parsers: vec![
//             Box::new(LambdayVarParser {}),
//         ] }.parse(input)
//     }
// }
//
// struct LambdayVarParser {}
// impl<'a, I: Input<InputElement=char>> Parser<I, LambdayTerm<&'a str>> for LambdayVarParser {
//     fn parse(&self, input: I) -> Result<ParseSuccess<I, LambdayTerm<&'a str>>, ParseError<I>> {
//
//     }
// }
