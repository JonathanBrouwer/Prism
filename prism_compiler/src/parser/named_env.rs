use prism_input::input::Input;
use prism_input::input_table::InputTable;
use prism_parser::env::GenericEnv;
use prism_parser::grammar::grammar_file::GrammarFile;
use prism_parser::parsable::parsed::Parsed;
use std::collections::HashMap;

#[derive(Default, Clone)]
pub struct NamedEnv {
    pub(crate) env_len: usize,
    pub names: NamesEnv,
    pub(crate) hygienic_names: GenericEnv<String, usize>,
}

pub type NamesEnv = GenericEnv<String, NamesEntry>;

#[derive(Clone)]
pub enum NamesEntry {
    FromEnv(usize),
    FromGrammarEnv {
        grammar_env_len: usize,
        adapt_env_len: usize,
        prev_env_len: usize,
    },
    FromParsed(Parsed, NamesEnv),
}

impl NamedEnv {
    pub fn insert_name(&self, name: &str, input: &InputTable) -> Self {
        let mut s = self.insert_name_at(name, self.env_len, input);
        s.env_len += 1;
        s
    }

    pub fn insert_name_at(&self, name: &str, depth: usize, input: &InputTable) -> Self {
        let names = self
            .names
            .insert(name.to_string(), NamesEntry::FromEnv(depth));
        let hygienic_names = if let Some(NamesEntry::FromParsed(ar, _)) = self.names.get(name) {
            let new_name = ar.value_ref::<Input>().as_str(input).to_string();
            self.hygienic_names.insert(new_name, depth)
        } else {
            self.hygienic_names.clone()
        };

        Self {
            env_len: self.env_len,
            names,
            hygienic_names,
        }
    }

    pub fn resolve_name_use(&self, name: &str) -> Option<&NamesEntry> {
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
        grammar: &GrammarFile,
        jump_labels: &mut HashMap<*const GrammarFile, NamesEnv>,
    ) {
        jump_labels.insert(grammar as *const GrammarFile, self.names.clone());
    }

    pub fn shift_back(&self, old_names: &NamesEnv, input: &InputTable) -> Self {
        let mut new_env = Self {
            env_len: self.env_len,
            names: old_names.clone(),
            //TODO what here? old code takes from `old_names` env (not available here)
            hygienic_names: Default::default(),
        };

        for (name, db_idx) in self.hygienic_names.iter() {
            new_env = new_env.insert_name_at(name, *db_idx, input);
        }

        new_env
    }
}
