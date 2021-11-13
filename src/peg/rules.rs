use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

pub trait TerminalPredicate<IE: Debug + Display + PartialEq + Eq + Clone + Copy, ER: Debug + Display + PartialEq + Eq + Clone + Copy> : Debug {
    fn run(&self, token: IE) -> bool;
    fn representitive(&self) -> ER;
}

#[derive(Debug, Clone)]
pub enum PegRule<IE: Debug + Display + PartialEq + Eq + Clone + Copy, ER: Debug + Display + PartialEq + Eq + Clone + Copy> {
    Terminal(Rc<dyn TerminalPredicate<IE, ER>>),
    Sequence(Vec<usize>),
    Choice(Vec<usize>),
}

pub type PegRuleResult<IE> = Rc<PegRuleResultInner<IE>>;

#[derive(Debug, Eq, PartialEq)]
pub enum PegRuleResultInner<IE: Debug + Display + PartialEq + Eq + Clone + Copy> {
    Terminal(usize, IE),
    Sequence(Vec<PegRuleResult<IE>>),
    Choice(usize, PegRuleResult<IE>),
}

impl<IE: Debug + Display + PartialEq + Eq + Clone + Copy> Display for PegRuleResultInner<IE> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PegRuleResultInner::Terminal(_, inp) => write!(f, "{}", inp),
            PegRuleResultInner::Sequence(seq) => {
                write!(f, "[")?;
                for (i, res) in seq.iter().enumerate() {
                    if i == 0 { write!(f, "{}", res)?; } else { write!(f, " {}", res)?; }
                }
                write!(f, "]")
            }
            PegRuleResultInner::Choice(i, res) => {
                write!(f, "<{} {}>", i, res)
            }
        }
    }
}