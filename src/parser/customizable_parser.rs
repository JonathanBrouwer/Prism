use petgraph::algo::DfsSpace;
use petgraph::prelude::*;
use petgraph::visit::{IntoNeighborsDirected, IntoNodeIdentifiers, Visitable, VisitMap};

use crate::lexer::lexer::{LexerItem, LexerToken, LexerTokenType};
use crate::lexer::lexer::LexerToken::{BlockStart, BlockEnd, Line};
use crate::parser::parser::*;
use std::ops::Deref;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ParseRule<'a> {
    Bind(LexerTokenType),
    Expect(LexerToken<'a>),
    SameLevelExpr,
    SubLevelExpr,
    Seq(Vec<ParseRule<'a>>)
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ParseGroup<'a> {
    pub rules: Vec<ParseRule<'a>>,
}

pub struct CustomizableParser<'a> {
    parse_groups: Vec<Vec<ParseGroup<'a>>>,
}

impl<'a> CustomizableParser<'a> {
    pub fn from_graph(mut graph: Graph<ParseGroup<'a>, ()>) -> Result<Self, ParseGroup<'a>> {
        let mut result: Vec<Vec<_>> = Vec::new();

        let mut visited = graph.visit_map();
        let mut current = Vec::new();

        for node in graph.node_indices() {
            if graph.neighbors_directed(node, Incoming).count() == 0 {
                current.push(node);
                visited.visit(node);
            }
        }

        let mut next = Vec::new();
        while !current.is_empty() {
            for &node in &current {
                for nb in graph.neighbors(node) {
                    if graph.neighbors_directed(nb, Incoming).filter(|nbb| !visited.is_visited(nbb)).count() == 0 {
                        if !next.contains(&nb) {
                            next.push(nb);
                        }
                    }
                    if graph.neighbors_directed(nb, Outgoing).any(|nbb| visited.is_visited(&nbb)) {
                        return Err(graph.node_weight(nb).unwrap().clone());
                    }
                }
            }
            for &node in &next {
                visited.visit(node);
            }

            result.push(current);
            current = next;
            next = Vec::new();
        }
        if let Some(v) = graph.node_indices().filter(|n| !visited.is_visited(n)).next() {
            return Err(graph.node_weight(v).unwrap().clone());
        }

        Ok(CustomizableParser::from_vec(result.into_iter().map(
            |sub| sub.into_iter().map(|n|
                graph.node_weight(n).unwrap().clone()
            ).collect()
        ).collect()
        ))
    }

    pub fn from_vec(parse_groups: Vec<Vec<ParseGroup<'a>>>) -> Self {
        Self { parse_groups }
    }

    pub fn parse(&self, input: &'a [LexerItem<'a>]) -> Result<ParseSuccess<'a, ()>, ParseError<'a>> {
        self.parse_sub(input, self.parse_groups.as_slice())
    }

    fn parse_sub(&self, input: &'a [LexerItem<'a>], groups: &[Vec<ParseGroup<'a>>]) -> Result<ParseSuccess<'a, ()>, ParseError<'a>> {
        assert!(groups.len() > 0);

        let mut error: Option<ParseError<'a>> = None;
        for g in &groups[0] {
            for r in &g.rules {
                match self.parse_rule(input, groups, r) {
                    Ok(v) => {
                        error = combine_err(error, v.best_error);
                        return Ok(ParseSuccess { result: v.result, rest: v.rest, best_error: error });
                    }
                    Err(e2) => error = match error {
                        Some(e1) => Some(e1.combine(e2)),
                        None => Some(e2),
                    }
                }
            }
        }
        return Err(error.unwrap());
    }

    fn parse_rule(&self, input: &'a [LexerItem<'a>], groups: &[Vec<ParseGroup<'a>>], rule: &ParseRule<'a>) -> Result<ParseSuccess<'a, ()>, ParseError<'a>> {
        Ok(match rule {
            ParseRule::Bind(v) => {
                expect_type(input, *v)?.map_result(|r| ())
            }
            ParseRule::Expect(v) => {
                expect_exact(input, *v)?.map_result(|r| ())
            }
            ParseRule::SameLevelExpr => {
                self.parse_sub(input, groups)?
            }
            ParseRule::SubLevelExpr => {
                self.parse_sub(input, &groups[1..])?
            }
            ParseRule::Seq(subrules) => {
                let mut rest = input;
                let mut best_error: Option<ParseError<'a>> = None;
                for subrule in subrules {
                    match self.parse_rule(rest, groups, subrule) {
                        Ok(suc) => {
                            rest = suc.rest;
                            best_error = combine_err(best_error, suc.best_error);
                        }
                        Err(fail) => {
                            best_error = combine_err(best_error, Some(fail));
                            return Err(best_error.unwrap())
                        }
                    }
                }
                ParseSuccess { result: (), best_error, rest }
            }
        })



        // let mut rest = input;
        // let mut best_error: Option<ParseError<'a>> = None;
        //
        // for p in &rule.parts {
        //     let res = match p {
        //         ParseRulePart::Bind(v) => {
        //             expect_type(rest, *v)?.map_result(|r| ())
        //         }
        //         ParseRulePart::Expect(v) => {
        //             expect_exact(rest, *v)?.map_result(|r| ())
        //         }
        //         ParseRulePart::SameLevelExpr => {
        //             self.parse_sub(rest, groups)?
        //         }
        //         ParseRulePart::SubLevelExpr => {
        //             self.parse_sub(rest, &groups[1..])?
        //         }
        //     };
        //     rest = res.rest;
        //     best_error = combine_err(best_error, res.best_error);
        // }
        // Ok(ParseSuccess{ result: (), rest, best_error })
    }
}

// #[cfg(test)]
// mod tests_init {
//     use petgraph::prelude::*;
//
//     use crate::parser::customizable_parser::*;
//
//     #[test]
//     fn test_from_graph1() {
//         let p1 = ParseGroup { rules: vec![ParseRule { parts: vec![ParseRulePart::Expect(LexerToken::Name("A"))] }] };
//         let p2 = ParseGroup { rules: vec![ParseRule { parts: vec![ParseRulePart::Expect(LexerToken::Name("B"))] }] };
//         let p3 = ParseGroup { rules: vec![ParseRule { parts: vec![ParseRulePart::Expect(LexerToken::Name("C"))] }] };
//         let p4 = ParseGroup { rules: vec![ParseRule { parts: vec![ParseRulePart::Expect(LexerToken::Name("D"))] }] };
//
//         let mut graph = Graph::new();
//         let p1i = graph.add_node(p1.clone());
//         let p2i = graph.add_node(p2.clone());
//         let p3i = graph.add_node(p3.clone());
//         let p4i = graph.add_node(p4.clone());
//
//         graph.add_edge(p1i, p3i, ());
//         graph.add_edge(p1i, p4i, ());
//         graph.add_edge(p2i, p3i, ());
//         graph.add_edge(p2i, p4i, ());
//         graph.add_edge(p1i, p2i, ());
//
//         let parser = CustomizableParser::from_graph(graph).unwrap();
//         assert_eq!(parser.parse_groups, vec![
//             vec![p1],
//             vec![p2],
//             vec![p4, p3]
//         ])
//     }
//
//     #[test]
//     fn test_from_graph2() {
//         let p1 = ParseGroup { rules: vec![ParseRule { parts: vec![ParseRulePart::Expect(LexerToken::Name("A"))] }] };
//         let p2 = ParseGroup { rules: vec![ParseRule { parts: vec![ParseRulePart::Expect(LexerToken::Name("B"))] }] };
//         let p3 = ParseGroup { rules: vec![ParseRule { parts: vec![ParseRulePart::Expect(LexerToken::Name("C"))] }] };
//         let p4 = ParseGroup { rules: vec![ParseRule { parts: vec![ParseRulePart::Expect(LexerToken::Name("D"))] }] };
//
//         let mut graph = Graph::new();
//         let p1i = graph.add_node(p1.clone());
//         let p2i = graph.add_node(p2.clone());
//         let p3i = graph.add_node(p3.clone());
//         let p4i = graph.add_node(p4.clone());
//
//         graph.add_edge(p1i, p2i, ());
//         graph.add_edge(p1i, p3i, ());
//         graph.add_edge(p1i, p4i, ());
//         graph.add_edge(p2i, p3i, ());
//         graph.add_edge(p2i, p4i, ());
//         graph.add_edge(p3i, p4i, ());
//
//
//         let parser = CustomizableParser::from_graph(graph).unwrap();
//         assert_eq!(parser.parse_groups, vec![
//             vec![p1],
//             vec![p2],
//             vec![p3],
//             vec![p4]
//         ])
//     }
//
//     #[test]
//     fn test_from_graph3() {
//         let p1 = ParseGroup { rules: vec![ParseRule { parts: vec![ParseRulePart::Expect(LexerToken::Name("A"))] }] };
//         let p2 = ParseGroup { rules: vec![ParseRule { parts: vec![ParseRulePart::Expect(LexerToken::Name("B"))] }] };
//         let p3 = ParseGroup { rules: vec![ParseRule { parts: vec![ParseRulePart::Expect(LexerToken::Name("C"))] }] };
//         let p4 = ParseGroup { rules: vec![ParseRule { parts: vec![ParseRulePart::Expect(LexerToken::Name("D"))] }] };
//
//         let mut graph = Graph::new();
//         let p1i = graph.add_node(p1.clone());
//         let p2i = graph.add_node(p2.clone());
//         let p3i = graph.add_node(p3.clone());
//         let p4i = graph.add_node(p4.clone());
//
//         graph.add_edge(p1i, p2i, ());
//         graph.add_edge(p2i, p3i, ());
//         graph.add_edge(p3i, p4i, ());
//
//
//         let parser = CustomizableParser::from_graph(graph).unwrap();
//         assert_eq!(parser.parse_groups, vec![
//             vec![p1],
//             vec![p2],
//             vec![p3],
//             vec![p4]
//         ])
//     }
//
//     #[test]
//     fn test_from_graph4() {
//         let p1 = ParseGroup { rules: vec![ParseRule { parts: vec![ParseRulePart::Expect(LexerToken::Name("A"))] }] };
//         let p2 = ParseGroup { rules: vec![ParseRule { parts: vec![ParseRulePart::Expect(LexerToken::Name("B"))] }] };
//         let p3 = ParseGroup { rules: vec![ParseRule { parts: vec![ParseRulePart::Expect(LexerToken::Name("C"))] }] };
//         let p4 = ParseGroup { rules: vec![ParseRule { parts: vec![ParseRulePart::Expect(LexerToken::Name("D"))] }] };
//
//         let mut graph = Graph::new();
//         let p1i = graph.add_node(p1.clone());
//         let p2i = graph.add_node(p2.clone());
//         let p3i = graph.add_node(p3.clone());
//         let p4i = graph.add_node(p4.clone());
//
//         graph.add_edge(p1i, p2i, ());
//         graph.add_edge(p1i, p3i, ());
//         graph.add_edge(p2i, p4i, ());
//         graph.add_edge(p3i, p4i, ());
//
//
//         let parser = CustomizableParser::from_graph(graph).unwrap();
//         assert_eq!(parser.parse_groups, vec![
//             vec![p1],
//             vec![p3, p2],
//             vec![p4]
//         ])
//     }
//
//     #[test]
//     fn test_from_cycle() {
//         let p1 = ParseGroup { rules: vec![ParseRule { parts: vec![ParseRulePart::Expect(LexerToken::Name("A"))] }] };
//         let p2 = ParseGroup { rules: vec![ParseRule { parts: vec![ParseRulePart::Expect(LexerToken::Name("B"))] }] };
//         let p3 = ParseGroup { rules: vec![ParseRule { parts: vec![ParseRulePart::Expect(LexerToken::Name("C"))] }] };
//         let p4 = ParseGroup { rules: vec![ParseRule { parts: vec![ParseRulePart::Expect(LexerToken::Name("D"))] }] };
//
//         let mut graph = Graph::new();
//         let p1i = graph.add_node(p1.clone());
//         let p2i = graph.add_node(p2.clone());
//         let p3i = graph.add_node(p3.clone());
//         let p4i = graph.add_node(p4.clone());
//
//         graph.add_edge(p1i, p2i, ());
//         graph.add_edge(p2i, p3i, ());
//         graph.add_edge(p3i, p4i, ());
//         graph.add_edge(p4i, p3i, ());
//
//
//         assert!(CustomizableParser::from_graph(graph).is_err())
//     }
// }
//
// #[cfg(test)]
// mod tests_parse {
//     use crate::parser::customizable_parser::*;
// }