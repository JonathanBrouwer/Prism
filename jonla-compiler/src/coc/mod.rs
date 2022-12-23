use rpds::Vector;

pub enum Expr {
    Type,
    Let(Box<Expr>, Box<Expr>),
    Var(usize),
    FnType(Box<Expr>, Box<Expr>),
    FnConstruct(Box<Expr>, Box<Expr>),
    FnDestruct(Box<Expr>, Box<Expr>),
}

type Env = Vector<Expr>;