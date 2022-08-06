use crate::formatting_file::FormattingFile;
use crate::grammar::{Rule, RuleBody};
use std::collections::HashMap;
use std::io::Write;

pub fn write_rules<'input>(mut file: FormattingFile, rules: &Vec<Rule<'input>>) {
    let map: HashMap<&'input str, Vec<RuleBody<'input>>> =
        rules.iter().map(|r| (r.name, r.body.clone())).collect();
    let json = serde_json::to_string(&map).unwrap();
    file.write(&json.as_bytes()).unwrap();
}
