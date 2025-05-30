use crate::lang::CoreIndex;
use crate::lang::PrismDb;
use prism_parser::env::GenericEnv;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct UniqueVariableId(usize);

impl PrismDb {
    pub fn new_tc_id(&mut self) -> UniqueVariableId {
        let id = UniqueVariableId(self.tc_id);
        self.tc_id += 1;
        id
    }
}

pub type DbEnv = GenericEnv<(), EnvEntry>;

#[derive(Clone, Debug)]
pub enum EnvEntry {
    // Definitions used during type checking
    /// We know the type of this variable, but not its value. The type is the second `UnionIndex`
    CType(UniqueVariableId, CoreIndex),
    CSubst(CoreIndex, CoreIndex),

    // Definitions used during beta reduction
    RType(UniqueVariableId),
    RSubst(CoreIndex, DbEnv),
}
