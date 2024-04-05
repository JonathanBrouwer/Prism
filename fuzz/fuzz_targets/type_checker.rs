#![no_main]

use libfuzzer_sys::fuzz_target;
use prism_compiler::coc::arbitrary::ExprWithEnv;

fuzz_target!(|expr: ExprWithEnv| {
    let mut expr = expr;
    let _ = expr.0.type_check(expr.1);
});
