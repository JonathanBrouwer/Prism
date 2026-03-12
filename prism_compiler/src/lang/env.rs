use crate::lang::CoreIndex;
use crate::type_check::UniqueVariableId;
use prism_data_structures::generic_env::List;
use prism_input::span::Span;

pub type PrismEnv = List<EnvEntry>;

#[derive(Clone, Debug)]
pub enum EnvEntry {
    // Definitions used during type checking
    /// We know the type of this variable, but not its value. The type is the second `UnionIndex`
    CType(UniqueVariableId, CoreIndex, Option<Span>),
    CSubst(CoreIndex, CoreIndex, Option<Span>),

    // Definitions used during beta reduction
    RType(UniqueVariableId),
    RSubst(CoreIndex, PrismEnv),
}

impl EnvEntry {
    pub fn name(&self) -> Option<Span> {
        match self {
            EnvEntry::CType(_, _, name) | EnvEntry::CSubst(_, _, name) => *name,
            EnvEntry::RSubst(..) | EnvEntry::RType(..) => unreachable!(),
        }
    }
}
