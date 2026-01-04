use crate::env::GenericEnv;
use crate::parsable::parsed::Parsed;

pub mod apply_action;
pub mod instance;
pub mod layout;
pub mod parsed_list;
pub mod placeholder_store;
pub mod recovery;
pub mod rule;
pub mod rule_block;
pub mod rule_closure;
pub mod rule_expr;

pub type VarMap = GenericEnv<String, Parsed>;
