use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::RuleExpr;
use crate::parser2::parse_sequence::ParserSequence;
use crate::parser2::{PResult, ParserChoiceSub, ParserState};
use crate::core::adaptive::BlockState;
use crate::parser2::add_rule::BlockCtx;

impl<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> ParserState<'arn, 'grm, E> {
    pub fn print_debug_info(&self) {
        print!("[{}] ", self.sequence_state.pos);
        for s in &self.sequence_stack {
            match s {
                ParserSequence::Exprs(es, _) => {
                    for e in es.iter().rev() {
                        print_debug_expr(e);
                        print!(" ");
                    }
                }
                ParserSequence::Blocks(bs) => {
                    print!("block{{{}}}", bs[0].name);
                },
                ParserSequence::PopChoice(_) => print!("pop"),
                ParserSequence::Repeat { expr, .. } => {
                    print!("repeat{{");
                    print_debug_expr(expr);
                    print!("}}");
                },
                ParserSequence::LeftRecurse { .. } => print!("lr"),
                ParserSequence::PositiveLookaheadEnd { .. } => print!("pl"),
                ParserSequence::NegativeLookaheadEnd { .. } => print!("ng"),
            }
            print!(" ");
        }

        print!("/ ");
        for c in &self.choice_stack {
            match c.choice {
                ParserChoiceSub::Blockss(bs) => {
                    print!("block{{{}}}", bs[0].name);
                },
                ParserChoiceSub::Constructors(cs, _) => print!("cs"),
                ParserChoiceSub::Exprs(exprs, _) => print!("exprs"),
                ParserChoiceSub::RepeatOptional => print!("rp"),
                ParserChoiceSub::NegativeLookaheadFail => print!("nl"),
                ParserChoiceSub::LeftRecursionFail => print!("lr"),
            }
            print!(" ");
        }

        println!();
    }
}

fn print_debug_expr(expr: &RuleExpr) {
    match expr {
        RuleExpr::RunVar(_, _) => todo!(),
        RuleExpr::CharClass(_) => todo!(),
        RuleExpr::Literal(lit) => print!("'{}'", lit.chars().collect::<String>()),
        RuleExpr::Repeat { .. } => todo!(),
        RuleExpr::Sequence(seq) => {
            print!("seq{{ ");
            for e in seq.iter() {
                print_debug_expr(e);
                print!("  ");
            }
            print!("}}");
        },
        RuleExpr::Choice(_) => todo!(),
        RuleExpr::NameBind(_, e) => print_debug_expr(e),
        RuleExpr::Action(e, _) => print_debug_expr(e),
        RuleExpr::SliceInput(_) => todo!(),
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
        print_debug_expr(&constructor.0.1);
        print!(" ");
    }
    print!("}}");
}