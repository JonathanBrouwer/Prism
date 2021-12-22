use std::fmt::{Debug, Display};
use itertools::Itertools;

pub trait Input: Clone + Debug {
    type InputElement: Debug + Display + PartialEq + Eq + Clone + Copy;

    fn next(&self) -> Option<(Self::InputElement, Self)>;
    fn pos(&self) -> usize;

    fn src_str<'a>(&'a self) -> Box<dyn ToString + 'a>;
    fn src_slice(&self) -> (usize, usize);
}

impl Input for (&str, usize) {
    type InputElement = char;

    fn next(&self) -> Option<(Self::InputElement, Self)> {
        if self.1 < self.0.len() {
            let c = self.0[self.1..].chars().next().unwrap();
            Some((c, (self.0, self.1 + c.len_utf8())))
        } else {
            None
        }
    }

    fn pos(&self) -> usize {
        self.1
    }

    fn src_str<'a>(&'a self) -> Box<dyn ToString + 'a> {
        Box::new(self.0)
    }

    fn src_slice(&self) -> (usize, usize) {
        (self.1, 1)
    }
}
