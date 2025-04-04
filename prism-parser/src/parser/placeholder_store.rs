use crate::core::allocs::Allocs;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::parsable::parsable_dyn::ParsableDyn;
use crate::parsable::parsed::Parsed;

#[derive(Copy, Clone, Debug)]
pub struct ParsedPlaceholder(usize);

struct StoreEntry<'arn, Env> {
    value: Option<Parsed<'arn>>,
    parent: Option<ParsedPlaceholder>,
    construct_info: Option<StoreEntryConstructInfo<'arn, Env>>,
}

struct StoreEntryConstructInfo<'arn, Env> {
    children: Vec<ParsedPlaceholder>,
    children_left: usize,
    constructor: &'arn str,
    parsable_dyn: ParsableDyn<'arn, Env>,
}

pub struct PlaceholderStore<'arn, Env> {
    store: Vec<StoreEntry<'arn, Env>>,
}

impl<'arn, Env> PlaceholderStore<'arn, Env> {
    pub fn push_empty(&mut self) -> ParsedPlaceholder {
        let len = self.store.len();
        self.store.push(StoreEntry {
            value: None,
            parent: None,
            construct_info: None,
        });
        ParsedPlaceholder(len)
    }

    pub fn place_construct_info(
        &mut self,
        cur: ParsedPlaceholder,
        constructor: &'arn str,
        parsable_dyn: ParsableDyn<'arn, Env>,
        children: Vec<ParsedPlaceholder>,
    ) {
        // Store info in children
        for child in &children {
            assert!(self.store[child.0].parent.is_none());
            self.store[child.0].parent = Some(cur);
        }

        // Store info in node
        let entry = &mut self.store[cur.0];
        entry.construct_info = Some(StoreEntryConstructInfo {
            children_left: children.len(),
            children,
            constructor,
            parsable_dyn,
        });
    }

    pub fn get(&self, index: ParsedPlaceholder) -> Option<Parsed<'arn>> {
        self.store[index.0].value
    }

    pub fn place_into_empty(
        &mut self,
        cur: ParsedPlaceholder,
        value: Parsed<'arn>,
        span: Span,
        allocs: Allocs<'arn>,
        src: &InputTable<'arn>,
        env: &mut Env,
    ) {
        // Store value
        let cur = &mut self.store[cur.0];
        assert!(cur.value.is_none());
        cur.value = Some(value);

        // Rest of this function is to update the parent if needed
        let Some(parent_idx) = cur.parent else { return };

        // Resolve parent
        let parent = &mut self.store[parent_idx.0].construct_info.as_mut().unwrap();

        // Update children left, break if there are
        parent.children_left -= 1;
        if parent.children_left != 0 {
            return;
        }

        // Construct value
        let parent = self.store[parent_idx.0].construct_info.as_ref().unwrap();
        let args = parent
            .children
            .iter()
            .map(|c| self.store[c.0].value.unwrap())
            .collect::<Vec<_>>();
        let value =
            (parent.parsable_dyn.from_construct)(span, parent.constructor, &args, allocs, src, env);

        // Place value, which will recurse if needed
        self.place_into_empty(parent_idx, value, span, allocs, src, env);
    }
}

impl<Env> Default for PlaceholderStore<'_, Env> {
    fn default() -> Self {
        Self { store: Vec::new() }
    }
}
