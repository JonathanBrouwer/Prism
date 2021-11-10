use std::fmt::{Debug, Display};

pub trait Input: Clone {
    type InputElement: Debug + Display + PartialEq + Eq + Clone + Copy;

    fn next(&self, from: usize) -> Option<(Self::InputElement, usize)>;
}

impl<T: Debug + Display + PartialEq + Eq + Clone + Copy> Input for &[T] {
    type InputElement = T;

    fn next(&self, from: usize) -> Option<(Self::InputElement, usize)> {
        if from < self.len() { Some((self[from], from + 1)) } else { None }
    }
}

impl<T: Debug + Display + PartialEq + Eq + Clone + Copy, const N: usize> Input for &[T; N] {
    type InputElement = T;

    fn next(&self, from: usize) -> Option<(Self::InputElement, usize)> {
        if from < self.len() { Some((self[from], from + 1)) } else { None }
    }
}

impl Input for &str {
    type InputElement = char;

    fn next(&self, from: usize) -> Option<(Self::InputElement, usize)> {
        if from < self.len() { Some((self[from..].chars().next().unwrap(), self.char_indices().skip(1).next().map(|i| i.0).unwrap_or(self.len()))) } else { None }
    }
}