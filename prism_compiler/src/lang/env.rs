use crate::lang::CoreIndex;
use crate::type_check::UniqueVariableId;
use prism_data_structures::generic_env::GenericEnv;

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
