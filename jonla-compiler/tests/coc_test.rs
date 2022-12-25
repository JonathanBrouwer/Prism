use jonla_compiler::coc::EnvEntry::NType;
use jonla_compiler::coc::Expr::*;
use jonla_compiler::coc::{brh, tc, Env, Expr, W};

fn assert_type(e: Expr, t: Expr) {
    assert_eq!(tc(&e, &Env::new()), Ok(t))
}

#[test]
fn test_tc1() {
    assert_type(Type, Type);
}

#[test]
fn test_tc2() {
    assert_type(
        FnConstruct(W::new(Type), W::new(Type)),
        FnType(W::new(Type), W::new(Type)),
    );
}

#[test]
fn test_tc3() {
    assert_type(FnType(W::new(Type), W::new(Type)), Type);
}

#[test]
fn test_tc4() {
    assert_type(
        FnConstruct(
            W::new(Type),
            W::new(FnConstruct(W::new(Var(0)), W::new(Var(1)))),
        ),
        FnType(W::new(Type), W::new(FnType(W::new(Var(0)), W::new(Type)))),
    );
}

#[test]
fn test_tc5() {
    assert_type(
        Let(
            W::new(FnConstruct(W::new(Type), W::new(Type))),
            W::new(Var(0)),
        ),
        FnType(W::new(Type), W::new(Type)),
    );
}

#[test]
fn test_tc6() {
    assert_type(
        Let(
            W::new(Type),
            W::new(Let(
                W::new(FnConstruct(W::new(Var(0)), W::new(Var(1)))),
                W::new(Var(0)),
            )),
        ),
        FnType(W::new(Type), W::new(Type)),
    );
}

#[test]
fn test_tc7() {
    assert_type(
        FnConstruct(
            W::new(Type),
            W::new(Let(
                W::new(FnConstruct(W::new(Var(0)), W::new(Var(0)))),
                W::new(Var(0)),
            )),
        ),
        FnType(W::new(Type), W::new(FnType(W::new(Var(0)), W::new(Var(1))))),
    )
}

#[test]
fn test_tc8() {
    assert_type(
        FnConstruct(
            W::new(Type),
            W::new(FnConstruct(W::new(Var(0)), W::new(Var(0)))),
        ),
        FnType(W::new(Type), W::new(FnType(W::new(Var(0)), W::new(Var(1))))),
    )
}

#[test]
fn test_tc9() {
    assert_type(
        FnConstruct(
            W::new(Type),
            W::new(FnDestruct(
                W::new(FnConstruct(W::new(Type), W::new(Var(0)))),
                W::new(Var(0)),
            )),
        ),
        FnType(W::new(Type), W::new(Type)),
    )
}

#[test]
fn test_tc10() {
    assert_type(
        FnConstruct(
            W::new(Type),
            W::new(FnDestruct(
                W::new(FnConstruct(W::new(Type), W::new(Var(1)))),
                W::new(Var(0)),
            )),
        ),
        FnType(W::new(Type), W::new(Type)),
    )
}

#[test]
fn test_tc11() {
    assert_type(
        FnConstruct(
            W::new(Type),
            W::new(FnConstruct(W::new(Var(0)), W::new(Var(0)))),
        ),
        FnType(W::new(Type), W::new(FnType(W::new(Var(0)), W::new(Var(1))))),
    )
}

#[test]
fn test_tc12() {
    assert_type(
        FnDestruct(
            W::new(FnConstruct(
                W::new(Type),
                W::new(FnConstruct(W::new(Var(0)), W::new(Var(0)))),
            )),
            W::new(Type),
        ),
        FnType(W::new(Type), W::new(Type)),
    )
}

#[test]
fn test_tc13() {
    // it "13" $ brh ([NType Type], Let (Var 0) (Var 0)) `shouldBe` ([NType Type], Var 0)

    assert_eq!(
        brh((
            &Let(W::new(Var(0)), W::new(Var(0)),),
            Env::new().cons(NType(&Type))
        )),
        (&Var(0), Env::new().cons(NType(&Type)))
    );
}

#[test]
fn test_tc15() {
    assert_type(
        Let(
            W::new(FnConstruct(
                W::new(Type),
                W::new(FnConstruct(W::new(Var(0)), W::new(Var(0)))),
            )),
            W::new(Var(0)),
        ),
        FnType(W::new(Type), W::new(FnType(W::new(Var(0)), W::new(Var(1))))),
    )
}

#[test]
fn test_tc16() {
    assert_type(
        Let(
            W::new(Type),
            W::new(FnConstruct(W::new(Var(0)), W::new(Var(0)))),
        ),
        FnType(W::new(Type), W::new(Type)),
    )
}
