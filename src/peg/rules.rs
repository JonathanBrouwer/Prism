use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

pub trait TerminalPredicate<IE: Debug + Display + PartialEq + Eq + Clone + Copy> : Debug {
    fn run(&self, token: IE) -> bool;
    fn representitive(&self) -> IE;
}

#[derive(Debug)]
pub struct ExactPredicate<IE: Debug + Display + PartialEq + Eq + Clone + Copy> {
    token: IE
}

impl<IE: Debug + Display + PartialEq + Eq + Clone + Copy> TerminalPredicate<IE> for ExactPredicate<IE> {
    fn run(&self, token: IE) -> bool {
        self.token == token
    }

    fn representitive(&self) -> IE {
        self.token
    }
}

pub struct Preds;
impl Preds {
    pub fn exact<IE: 'static +  Debug + Display + PartialEq + Eq + Clone + Copy>(token: IE) -> Rc<dyn TerminalPredicate<IE>> {
        Rc::new(ExactPredicate{ token })
    }
}

#[derive(Debug, Clone)]
pub enum PegRule<IE: Debug + Display + PartialEq + Eq + Clone + Copy> {
    Terminal(Rc<dyn TerminalPredicate<IE>>),
    Sequence(Vec<usize>),
    Choice(Vec<usize>),
}

pub type PegRuleResult<IE> = Rc<PegRuleResultInner<IE>>;

#[derive(Debug, Eq, PartialEq)]
pub enum PegRuleResultInner<IE: Debug + Display + PartialEq + Eq + Clone + Copy> {
    Terminal(IE),
    Sequence(Vec<PegRuleResult<IE>>),
    Choice(usize, PegRuleResult<IE>),
}

impl<IE: Debug + Display + PartialEq + Eq + Clone + Copy> Display for PegRuleResultInner<IE> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PegRuleResultInner::Terminal(inp) => write!(f, "{}", inp),
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