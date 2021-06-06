use crate::peg_parser::peg_parser::*;
use petgraph::prelude::*;
use petgraph::{Graph, Directed, Direction};
use std::collections::{VecDeque, HashSet};
use crate::peg_parser::grammar_analysis::AllAny::*;
use petgraph::visit::VisitMap;

pub fn analyse_grammar<TT: TokenType, T: Token<TT>>(rules: &Vec<PegRule<TT, T>>) -> Vec<bool> {
    let empty = empty_analysis(rules);
    let leftrec = leftrec_analysis(rules, empty);
    return leftrec
}

pub fn leftrec_analysis<TT: TokenType, T: Token<TT>>(rules: &Vec<PegRule<TT, T>>, empty_analysis: Vec<bool>) -> Vec<bool> {
    let mut graph = Graph::new();

    //Add nodes
    let nodes: Vec<NodeIndex> = rules.iter().map(|rule| {
        graph.add_node(())
    }).collect();

    //Add edges
    nodes.iter().zip(rules.iter()).for_each(|(from, rule)| {
        match rule {
            PegRule::LiteralExact(_) => {}
            PegRule::LiteralBind(_) => {}
            PegRule::Sequence(seq) => {
                for &to in seq.iter() {
                    graph.add_edge(*from, nodes[to], ());
                    if !empty_analysis[to] { break }
                }
            }
            PegRule::ChooseFirst(seq) => {seq.iter().for_each(|to| { graph.add_edge(*from, nodes[*to], ()); }); }
            PegRule::Repeat(to, _, _) => {graph.add_edge(*from, nodes[*to], ()); }
            PegRule::Option(to) => {graph.add_edge(*from, nodes[*to], ());}
            PegRule::LookaheadPositive(to) => {graph.add_edge(*from, nodes[*to], ());}
            PegRule::LookaheadNegative(to) => {graph.add_edge(*from, nodes[*to], ());}
        }
    });

    //Find leftrec
    let mut result = vec![false; graph.node_count()];
    for node in nodes {
        result[node.index()] = leftrec_check(&graph, node);
    }
    result
}

pub fn leftrec_check(graph: &Graph<(), ()>, from: NodeIndex) -> bool {
    let mut dfs = Dfs::new(&graph, from);
    while let Some(node) = dfs.stack.pop() {
        if dfs.discovered.visit(node) {
            for succ in graph.neighbors(node) {
                if !dfs.discovered.is_visited(&succ) {
                    dfs.stack.push(succ);
                } else if succ == from {
                    return true;
                }
            }
        }
    }
    return false;
}

pub fn empty_analysis<TT: TokenType, T: Token<TT>>(rules: &Vec<PegRule<TT, T>>) -> Vec<bool> {
    let mut graph = Graph::new();

    //Add nodes
    let nodes: Vec<NodeIndex> = rules.iter().map(|rule| {
        let node_type = match rule {
            PegRule::LiteralExact(_) => Never,
            PegRule::LiteralBind(_) => Never,
            PegRule::Sequence(_) => All,
            PegRule::ChooseFirst(_) => Any,
            PegRule::Repeat(_, min, _) => if min.unwrap_or(0) == 0 { AllAny::Always } else { AllAny::All },
            PegRule::Option(_) => Always,
            PegRule::LookaheadPositive(_) => Always,
            PegRule::LookaheadNegative(_) => Always,
        };
        graph.add_node(node_type)
    }).collect();

    //Add edges
    nodes.iter().zip(rules.iter()).for_each(|(from, rule)| {
       match rule {
           PegRule::LiteralExact(_) => {}
           PegRule::LiteralBind(_) => {}
           PegRule::Sequence(seq) => {seq.iter().for_each(|to| { graph.add_edge(*from, nodes[*to], ()); }); }
           PegRule::ChooseFirst(seq) => {seq.iter().for_each(|to| { graph.add_edge(*from, nodes[*to], ()); }); }
           PegRule::Repeat(to, _, _) => {graph.add_edge(*from, nodes[*to], ()); }
           PegRule::Option(_) => {}
           PegRule::LookaheadPositive(_) => {}
           PegRule::LookaheadNegative(_) => {}
       }
    });

    all_any_analyser(graph)
}

pub enum AllAny {
    Always, All, Any, Never
}

pub fn all_any_analyser(graph: Graph<AllAny, (), Directed>) -> Vec<bool> {
    let mut result = vec![false; graph.node_count()];

    let mut queue = VecDeque::new();
    graph.node_indices().for_each(|i| { queue.push_back(i); });

    while !queue.is_empty() {
        let next = queue.pop_back().unwrap();

        if result[next.index()] { continue }
        if match graph.node_weight(next).unwrap() {
            AllAny::Always => true,
            AllAny::All => graph.edges_directed(next, Direction::Outgoing).all(|e| result[e.target().index()]),
            AllAny::Any => graph.edges_directed(next, Direction::Outgoing).any(|e| result[e.target().index()]),
            AllAny::Never => false
        } {
            result[next.index()] = true;
            graph.edges_directed(next, Direction::Incoming).for_each(|e| { queue.push_back(e.source()); });
        }

    }

    result
}