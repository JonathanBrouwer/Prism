use crate::lang::CoreIndex;
use crate::lang::PrismEnv;
use prism_parser::env::GenericEnv;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct UniqueVariableId(usize);

impl<'arn> PrismEnv<'arn> {
    pub fn new_tc_id(&mut self) -> UniqueVariableId {
        let id = UniqueVariableId(self.tc_id);
        self.tc_id += 1;
        id
    }
}

pub type DbEnv<'arn> = GenericEnv<'arn, (), EnvEntry<'arn>>;

#[derive(Copy, Clone, Debug)]
pub enum EnvEntry<'arn> {
    // Definitions used during type checking
    /// We know the type of this variable, but not its value. The type is the second `UnionIndex`
    CType(UniqueVariableId, CoreIndex),
    CSubst(CoreIndex, CoreIndex),

    // Definitions used during beta reduction
    RType(UniqueVariableId),
    RSubst(CoreIndex, DbEnv<'arn>),
}
