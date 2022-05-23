let wrapper : Type -> Type = / p:Type. (c : Type) -> (p -> c) -> c
let proj : (p : Type) -> wrapper p -> p = / p : Type, a : wrapper p. a p (/ x : p. x)
proj