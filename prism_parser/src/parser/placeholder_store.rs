use crate::core::input::Input;
use crate::parsable::parsable_dyn::ParsableDyn;
use crate::parsable::parsed::Parsed;
use prism_input::span::Span;

#[derive(Copy, Clone, Debug)]
pub struct ParsedPlaceholder(usize);

struct StoreEntry<Db> {
    value: Option<Parsed>,
    parent: Option<ParsedPlaceholder>,
    construct_info: Option<StoreEntryConstructInfo<Db>>,
}

struct StoreEntryConstructInfo<Db> {
    children: Vec<ParsedPlaceholder>,
    children_left: usize,
    constructor: Input,
    parsable_dyn: ParsableDyn<Db>,
}

pub struct PlaceholderStore<Db> {
    store: Vec<StoreEntry<Db>>,
}

impl<Db> PlaceholderStore<Db> {
    pub fn push_empty(&mut self) -> ParsedPlaceholder {
        let len = self.store.len();
        self.store.push(StoreEntry {
            value: None,
            parent: None,
            construct_info: None,
        });
        ParsedPlaceholder(len)
    }

    pub fn get(&self, index: ParsedPlaceholder) -> Option<&Parsed> {
        self.store[index.0].value.as_ref()
    }

    pub fn place_construct_info(
        &mut self,
        cur: ParsedPlaceholder,
        constructor: Input,
        parsable_dyn: ParsableDyn<Db>,
        children: Vec<ParsedPlaceholder>,
        env: &mut Db,
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

        self.bubble_up(cur, env);
    }

    pub fn place_into_empty(&mut self, cur: ParsedPlaceholder, value: Parsed, env: &mut Db) {
        // Store value
        let cur = &mut self.store[cur.0];
        assert!(cur.value.is_none());
        cur.value = Some(value);

        // Rest of this function is to update the parent if needed
        let Some(parent_idx) = cur.parent else { return };

        let parent = &mut self.store[parent_idx.0].construct_info.as_mut().unwrap();
        parent.children_left -= 1;

        self.bubble_up(parent_idx, env);
    }

    fn bubble_up(&mut self, parent_idx: ParsedPlaceholder, env: &mut Db) {
        // Resolve parent
        let parent = &mut self.store[parent_idx.0].construct_info.as_mut().unwrap();

        // Update children left, break if there are
        if parent.children_left != 0 {
            return;
        }

        // Construct value
        let parent = self.store[parent_idx.0].construct_info.as_ref().unwrap();
        let args = parent
            .children
            .iter()
            .map(|c| self.store[c.0].value.as_ref().unwrap().clone())
            .collect::<Vec<_>>();

        let span = Span::dummy();
        let value = (parent.parsable_dyn.from_construct)(span, &parent.constructor, &args, env);

        // Place value, which will recurse if needed
        self.place_into_empty(parent_idx, value, env);
    }
}

impl<Db> Default for PlaceholderStore<Db> {
    fn default() -> Self {
        Self { store: Vec::new() }
    }
}
