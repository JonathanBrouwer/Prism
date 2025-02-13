use crate::parsable::ParseResult;
use crate::parsable::parsed::Parsed;
use crate::parsable::void::Void;
use std::ops::{Index, IndexMut};

#[derive(Copy, Clone)]
pub struct ParsedPlaceholder(usize);

struct StoreEntry<'arn, 'grm> {
    value: Parsed<'arn, 'grm>,
}

#[derive(Default)]
pub struct PlaceholderStore<'arn, 'grm> {
    store: Vec<StoreEntry<'arn, 'grm>>,
}

impl<'arn, 'grm> PlaceholderStore<'arn, 'grm> {
    pub fn push(
        &mut self,
        constructor: &'grm str,
        children: &[ParsedPlaceholder],
    ) -> ParsedPlaceholder {
        let len = self.store.len();
        self.store.push(StoreEntry {
            value: Void.to_parsed(),
        });
        ParsedPlaceholder(len)
    }
}

impl<'arn, 'grm> Index<ParsedPlaceholder> for PlaceholderStore<'arn, 'grm> {
    type Output = Parsed<'arn, 'grm>;

    fn index(&self, index: ParsedPlaceholder) -> &Self::Output {
        &self.store[index.0].value
    }
}

impl<'arn, 'grm> IndexMut<ParsedPlaceholder> for PlaceholderStore<'arn, 'grm> {
    fn index_mut(&mut self, index: ParsedPlaceholder) -> &mut Self::Output {
        &mut self.store[index.0].value
    }
}
