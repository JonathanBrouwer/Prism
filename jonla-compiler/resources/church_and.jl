let and : Type -> Type -> Type = / p:Type, q:Type. (c : Type) -> (p -> q -> c) -> c
let conj : (p : Type) -> (q : Type) -> p -> q -> and p q = / p : Type, q : Type, x : p, y : q, c : Type, f : p -> q -> c. f x y
let proj1 : (p : Type) -> (q : Type) -> and p q -> p = / p : Type, q : Type, a : and p q. a p (/ x : p, _ : q. x)
let proj2 : (p : Type) -> (q : Type) -> and p q -> q = / p : Type, q : Type, a : and p q. a q (/ _ : p, y : q. y)
