use prism_parser::core::allocs::Allocs;
use prism_parser::core::input::Input;
use prism_parser::core::input_table::InputTable;
use prism_parser::env::GenericEnv;
use prism_parser::grammar::grammar_file::GrammarFile;
use prism_parser::parsable::parsed::Parsed;
use std::collections::HashMap;

#[derive(Default, Clone, Copy)]
pub struct NamedEnv<'arn> {
    pub(crate) env_len: usize,
    pub names: NamesEnv<'arn>,
    pub(crate) hygienic_names: GenericEnv<'arn, &'arn str, usize>,
}

pub type NamesEnv<'arn> = GenericEnv<'arn, &'arn str, NamesEntry<'arn>>;

#[derive(Debug, Copy, Clone)]
pub enum NamesEntry<'arn> {
    FromEnv(usize),
    FromGrammarEnv {
        grammar_env_len: usize,
        adapt_env_len: usize,
        prev_env_len: usize,
    },
    FromParsed(Parsed<'arn>, NamesEnv<'arn>),
}

impl<'arn> NamedEnv<'arn> {
    pub fn insert_name(
        &self,
        name: &'arn str,
        input: &InputTable<'arn>,
        allocs: Allocs<'arn>,
    ) -> Self {
        let mut s = self.insert_name_at(name, self.env_len, input, allocs);
        s.env_len += 1;
        s
    }

    pub fn insert_name_at(
        &self,
        name: &'arn str,
        depth: usize,
        input: &InputTable<'arn>,
        allocs: Allocs<'arn>,
    ) -> Self {
        let names = self.names.insert(name, NamesEntry::FromEnv(depth), allocs);
        let hygienic_names = if let Some(NamesEntry::FromParsed(ar, _)) = self.names.get(name) {
            let new_name = ar.into_value::<Input>().as_str(input);
            self.hygienic_names.insert(new_name, depth, allocs)
        } else {
            self.hygienic_names
        };

        Self {
            env_len: self.env_len,
            names,
            hygienic_names,
        }
    }

    pub fn resolve_name_use(&self, name: &str) -> Option<NamesEntry<'arn>> {
        self.names.get(name)
    }

    pub fn len(&self) -> usize {
        self.env_len
    }

    pub fn is_empty(&self) -> bool {
        self.env_len == 0
    }

    pub fn insert_shift_label(
        &self,
        grammar: &'arn GrammarFile<'arn>,
        jump_labels: &mut HashMap<*const GrammarFile<'arn>, NamesEnv<'arn>>,
    ) {
        jump_labels.insert(grammar as *const _, self.names);
    }

    pub fn shift_back(
        &self,
        old_names: NamesEnv<'arn>,
        input: &InputTable<'arn>,
        allocs: Allocs<'arn>,
    ) -> Self {
        let mut new_env = Self {
            env_len: self.env_len,
            names: old_names,
            //TODO what here? old code takes from `old_names` env (not available here)
            hygienic_names: Default::default(),
        };

        for (name, db_idx) in self.hygienic_names {
            new_env = new_env.insert_name_at(name, db_idx, input, allocs);
        }

        new_env
    }
}
