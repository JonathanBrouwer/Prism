* Spans & error messages
* * Queued type constraints
* Maybe remove type from FnConstruct? Think about how to desugar
* Make parser non-recursive
* Think about how to do names
  * Built-in magic pass?
* Fast-mode for parser & type checker without errors
* Eta reduction
  * expect_beq `fn_destruct == fn_construct` is possible
* Linear type system
  * Some way to encode mutable references etc?
  * `0`/`1`/`copy` system
  * Maybe `-1` (or something like that) for values that are not even present at compile time
    * Like irrelevance in Agda https://agda.readthedocs.io/en/latest/language/irrelevance.html
  * `0` for values that can't are erased at runtime
  * `1` Idris has `exactly once`, we want this but we may also want
    * `at most once`, with auto-drop implementation (like File) 
    * `at most once`, without drop (like &mut)
  * Inspire by Idris 2! https://idris2.readthedocs.io/en/latest/tutorial/multiplicities.html
  * If types are present at runtime, allow matching on them https://idris2.readthedocs.io/en/latest/tutorial/multiplicities.html#pattern-matching-on-types

  * `1/` intrinsic to `types`
  * `irrelevant/compile time only/available at runtime` notation on `variables`

  * `copy` types are opt-out, everything is automatically copy unless you don't want it to be

* Visiblity:
  * 3 different types of visibility:
    * fully public: Both the signature and the implementation of the function are public. Changing anything about the implementation is a breaking change.
    * public signature: The signature is public, but the implementation is not (this is relevant for evaluation during type checking). In this mode, functions can be fully evaluated but not partially evaluated during type checking. This makes sure you can change the implementation and as long as the input-output behaviour is unchanged it's not a breaking change
    * private: Not visible, like in rust
  * Implemented through modules:
    * private not exported through module
    * public signature: type def exported through module
    * fully public: definition exported through module


PARSER REFACTOR TODO:
- Prevent it from being recursive
- bincode -> postcard?
- Refactor `variables` to be O(1) clone




