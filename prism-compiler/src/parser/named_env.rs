use crate::lang::CheckedIndex;
use prism_parser::core::allocs::Allocs;
use prism_parser::core::input::Input;
use prism_parser::env::GenericerEnv;
use prism_parser::parsable::guid::Guid;
use prism_parser::parsable::parsed::Parsed;
use std::collections::HashMap;

#[derive(Default, Clone)]
pub struct NamedEnv<'arn, 'grm> {
    pub(crate) env_len: usize,
    pub names: NamesEnv<'arn, 'grm>,
    pub(crate) hygienic_names: GenericerEnv<'arn, &'arn str, usize>,
}

pub type NamesEnv<'arn, 'grm> = GenericerEnv<'arn, &'arn str, NamesEntry<'arn, 'grm>>;

#[derive(Debug, Copy, Clone)]
pub enum NamesEntry<'arn, 'grm> {
    FromEnv(usize),
    FromEnvSubst(CheckedIndex),
    FromParsed(Parsed<'arn, 'grm>, NamesEnv<'arn, 'grm>),
}

impl<'arn, 'grm: 'arn> NamedEnv<'arn, 'grm> {
    pub fn insert_name(&self, name: &'arn str, input: &'arn str, allocs: Allocs<'arn>) -> Self {
        let mut s = if name != "_" {
            self.insert_name_at(name, self.env_len, input, allocs)
        } else {
            self.clone()
        };
        s.env_len += 1;
        s
    }

    pub fn insert_name_at(
        &self,
        name: &'arn str,
        depth: usize,
        input: &'arn str,
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

    pub fn resolve_name_use(&self, name: &str) -> Option<NamesEntry<'arn, 'grm>> {
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
        guid: Guid,
        jump_labels: &mut HashMap<Guid, NamesEnv<'arn, 'grm>>,
    ) {
        jump_labels.insert(guid, self.names);
    }

    pub fn shift_back(
        &self,
        old_names: NamesEnv<'arn, 'grm>,
        input: &'arn str,
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
