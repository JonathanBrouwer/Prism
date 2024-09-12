use crate::core::adaptive::BlockState;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::RuleExpr;
use crate::parser2::parse_sequence::ParserSequence;
use crate::parser2::{PResult, ParserChoiceSub, ParserState};

impl<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> ParserState<'arn, 'grm, E> {
    pub fn print_debug_info(&self) {
        print!("[{}] ", self.sequence_state.pos);
        for s in &self.sequence_stack {
            match s {
                ParserSequence::Exprs(es) => {
                    print!("exprs{{ ");
                    for e in es.iter() {
                        print_debug_expr(e);
                        print!(" ");
                    }
                    print!("}}");
                }
                ParserSequence::Blocks(bs) => {
                    print!("block{{{}}}", bs[0].name);
                }
                ParserSequence::PopChoice(_) => print!("pop"),
                ParserSequence::Repeat { expr, .. } => {
                    print!("repeat{{");
                    print_debug_expr(expr);
                    print!("}}");
                }
                ParserSequence::LeftRecurse { key, .. } => print!("lr{{{}}}", key.block.name),
                ParserSequence::PositiveLookaheadEnd { .. } => print!("pl"),
                ParserSequence::NegativeLookaheadEnd { .. } => print!("ng"),
                ParserSequence::Layout { .. } => print!("la"),
                ParserSequence::CharClass(_) => print!("cc"),
                ParserSequence::Literal(lit) => print!("'{}'", lit.chars().collect::<String>()),
            }
            print!(" ");
        }

        print!("/ ");
        for c in &self.choice_stack {
            match c.choice {
                ParserChoiceSub::Blocks(bs) => {
                    print!("block{{{}}}", bs[0].name);
                }
                ParserChoiceSub::Constructors(_) => print!("cs"),
                ParserChoiceSub::Exprs(_) => print!("exprs"),
                ParserChoiceSub::RepeatFail => print!("ref"),
                ParserChoiceSub::NegativeLookaheadFail => print!("nlf"),
                ParserChoiceSub::LeftRecursionFail => print!("lrf"),
                ParserChoiceSub::LayoutFail => print!("laf"),
            }
            print!(" ");
        }

        println!();
    }
}

fn print_debug_expr(expr: &RuleExpr) {
    match expr {
        RuleExpr::RunVar(v, _) => print!("rule{{{v}}}"),
        RuleExpr::CharClass(cc) => print!("cc"),
        RuleExpr::Literal(lit) => print!("'{}'", lit.chars().collect::<String>()),
        RuleExpr::Repeat { .. } => todo!(),
        RuleExpr::Sequence(seq) => {
            print!("seq{{ ");
            for e in seq.iter() {
                print_debug_expr(e);
                print!("  ");
            }
            print!("}}");
        }
        RuleExpr::Choice(_) => todo!(),
        RuleExpr::NameBind(_, e) => print_debug_expr(e),
        RuleExpr::Action(e, _) => print_debug_expr(e),
        RuleExpr::SliceInput(s) => {
            print!("slice{{");
            print_debug_expr(s);
            print!("}}");
        },
        RuleExpr::PosLookahead(_) => todo!(),
        RuleExpr::NegLookahead(_) => todo!(),
        RuleExpr::This => print!("#this"),
        RuleExpr::Next => print!("#next"),
        RuleExpr::AtAdapt(_, _) => todo!(),
        RuleExpr::Guid => todo!(),
    }
}

fn print_debug_block(block: &BlockState) {
    print!("block{{");
    for constructor in block.constructors {
        print_debug_expr(&constructor.0 .1);
        print!(" ");
    }
    print!("}}");
}
