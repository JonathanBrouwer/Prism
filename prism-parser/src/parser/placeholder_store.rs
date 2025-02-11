use crate::parsable::ParseResult;
use crate::parsable::parsed::Parsed;
use crate::parsable::void::Void;
use std::ops::{Index, IndexMut};

#[derive(Copy, Clone)]
pub struct ParsedPlaceholder(usize);

#[derive(Default)]
pub struct PlaceholderStore<'arn, 'grm> {
    store: Vec<Parsed<'arn, 'grm>>,
}

impl<'arn, 'grm> PlaceholderStore<'arn, 'grm> {
    pub fn push(&mut self) -> ParsedPlaceholder {
        let len = self.store.len();
        self.store.push(Void.to_parsed());
        ParsedPlaceholder(len)
    }
}

impl<'arn, 'grm> Index<ParsedPlaceholder> for PlaceholderStore<'arn, 'grm> {
    type Output = Parsed<'arn, 'grm>;

    fn index(&self, index: ParsedPlaceholder) -> &Self::Output {
        &self.store[index.0]
    }
}

impl<'arn, 'grm> IndexMut<ParsedPlaceholder> for PlaceholderStore<'arn, 'grm> {
    fn index_mut(&mut self, index: ParsedPlaceholder) -> &mut Self::Output {
        &mut self.store[index.0]
    }
}
