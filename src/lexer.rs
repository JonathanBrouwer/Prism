use logos::{Logos, Lexer, Span};
use crate::lexer::LexerToken::{Line, BlockStart, BlockStop};
use std::cmp::Ordering;

#[derive(Logos, Debug, PartialEq, Eq)]
pub enum LexerToken {
    #[token("(")]
    ParenOpen,

    #[token(")")]
    ParenClose,

    #[token("{")]
    BracketOpen,

    #[token("}")]
    BracketClose,

    #[regex(r"[\p{Letter}\p{Mark}\p{Symbol}\p{Number}\p{Dash_Punctuation}\p{Connector_Punctuation}\p{Other_Punctuation}]+")]
    Identifier,

    #[token("\n")]
    Line,

    BlockStart,
    BlockStop,

    #[error]
    #[regex(r"[\p{Separator}\r]+", logos::skip)]
    Error,
}

pub struct JonlaLexer<'a> {
    logos_lexer : Lexer<'a, LexerToken>,

    // Indent tracker
    after_line : bool,
    indent : usize
}

impl<'a> JonlaLexer<'a> {
    pub fn lexer(inp: &'a str) -> JonlaLexer<'a> {
        JonlaLexer { logos_lexer: LexerToken::lexer(inp) , after_line: true , indent: 0 }
    }
}

impl<'a> Iterator for JonlaLexer<'a> {
    type Item = (LexerToken, Span);

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.logos_lexer.span().end;

        if self.after_line {
            //Get new indent
            let rem: &str = self.logos_lexer.remainder();
            let space_count = rem.chars().take_while(|&c| c == ' ').count();

            //Check if indent is valid, calculate new indent
            if space_count % 4 != 0 {
                //TODO make a best guess on what the new indent level is?
                self.after_line = false;
                return Some( (LexerToken::Error, start..start+space_count) );
            }
            let indent_new = space_count / 4;

            //Do something based on indent change
            if indent_new == self.indent {
                self.after_line = false;
            } else if indent_new > self.indent {
                self.indent += 1;
                return Some( (BlockStart, start..start+space_count) )
            } else if indent_new < self.indent {
                self.indent -= 1;
                return Some( (BlockStop, start..start+space_count) )
            }
        }

        //Find next token
        let next = self.logos_lexer.next();

        //If this is a line token,
        if let Some(LexerToken::Line) = next {
            self.after_line = true;
        }
        //If this is the end of the file
        else if let None = next {
            if self.indent > 0 {
                self.indent -= 1;
                return Some( (BlockStop, start..start) )
            }
        }

        next.map(|l| (l, self.logos_lexer.span()))
    }
}