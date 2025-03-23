use crate::env::GenericEnv;
use crate::parsable::parsed::Parsed;

pub mod apply_action;
pub mod parsed_list;
pub mod parser_instance;
pub mod parser_layout;
pub mod parser_rule;
pub mod parser_rule_block;
pub mod parser_rule_expr;
pub mod placeholder_store;
pub mod rule_closure;

pub type VarMap<'arn> = GenericEnv<'arn, &'arn str, Parsed<'arn>>;
