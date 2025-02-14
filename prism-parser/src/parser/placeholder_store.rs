use crate::parsable::ParseResult;
use crate::parsable::parsable_dyn::ParsableDyn;
use crate::parsable::parsed::Parsed;
use crate::parsable::void::Void;
use std::ops::Index;

#[derive(Copy, Clone)]
pub struct ParsedPlaceholder(usize);

struct StoreEntry<'arn, 'grm, Env> {
    value: Option<Parsed<'arn, 'grm>>,
    children_left: usize,
    parent: Option<ParsedPlaceholder>,
    constructor: &'grm str,
    parsable_dyn: ParsableDyn<'arn, 'grm, Env>,
}

pub struct PlaceholderStore<'arn, 'grm, Env> {
    store: Vec<StoreEntry<'arn, 'grm, Env>>,
}

impl<'arn, 'grm, Env> PlaceholderStore<'arn, 'grm, Env> {
    pub fn push(
        &mut self,
        constructor: &'grm str,
        parsable_dyn: ParsableDyn<'arn, 'grm, Env>,
        children: &[ParsedPlaceholder],
    ) -> ParsedPlaceholder {
        let len = self.store.len();
        self.store.push(StoreEntry {
            value: None,
            children_left: children.len(),
            parent: None,
            constructor,
            parsable_dyn,
        });
        let v = ParsedPlaceholder(len);
        for child in children {
            self.store[child.0].parent = Some(v);
        }
        v
    }

    pub fn get(&self, index: ParsedPlaceholder) -> Option<Parsed<'arn, 'grm>> {
        self.store[index.0].value
    }

    pub fn store(&mut self, index: ParsedPlaceholder, value: Parsed<'arn, 'grm>) {
        let value_ref = &mut self.store[index.0].value;
        assert!(value_ref.is_none());
        *value_ref = Some(value);
    }
}

impl<'arn, 'grm, Env> Default for PlaceholderStore<'arn, 'grm, Env> {
    fn default() -> Self {
        Self { store: Vec::new() }
    }
}
