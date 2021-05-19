use crate::lexer::lexer::{LexerToken, LexerLine};
use std::cmp::Ordering;
use std::collections::VecDeque;
use crate::lexer::layout_builder::LayoutToken::{Block, Line};
use std::ops::Range;
use std::fmt::{Debug};
use std::fmt;

pub enum LayoutToken<'a> {
    Line(Vec<LexerToken<'a>>),
    Block(Vec<LayoutToken<'a>>)
}

impl<'a> Debug for LayoutToken<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Line(t) => {
                write!(f, "Line{:?}", t)
            },
            Block(v) => {
                writeln!(f, "{}", "Block[")?;
                for item in v {
                    writeln!(f, "{:?}", item)?;
                }
                write!(f, "{}", "]")
            }
        }
    }
}

pub struct LayoutBuilder<'a> {
    pub input: Vec<LexerLine<'a>>
}

impl<'a> LayoutBuilder<'a> {
    pub fn build_layout(self) -> (Vec<LayoutToken<'a>>, Vec<Range<usize>>) {
        let mut stack = VecDeque::new();
        stack.push_back((0usize, Vec::new()));

        let mut errors: Vec<Range<usize>> = Vec::new();

        for line in self.input {
            match line.indent.cmp(&stack.back().unwrap().0) {
                Ordering::Less => {
                    while line.indent < stack.back().unwrap().0 {
                        let pop = stack.pop_back().unwrap().1;
                        stack.back_mut().unwrap().1.push(Block(pop));
                    }
                    if line.indent != stack.back().unwrap().0 {
                        errors.push(line.tokens.first().unwrap().span.start .. line.tokens.last().unwrap().span.end);
                    }
                    stack.back_mut().unwrap().1.push(Line(line.tokens));
                }
                Ordering::Equal => {
                    stack.back_mut().unwrap().1.push(Line(line.tokens));
                }
                Ordering::Greater => {
                    stack.push_back((line.indent, vec![Line(line.tokens)]));
                }
            }
        }

        while stack.len() > 1 {
            let pop = stack.pop_back().unwrap().1;
            stack.back_mut().unwrap().1.push(Block(pop));
        }
        (stack.pop_back().unwrap().1, errors)
    }
}