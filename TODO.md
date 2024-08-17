THE BIG TODO LIST:
- Eta reduction
- Rewrite parser to be non-recursive
- LSP support
  - Allow attributes such as `#[lsp("comment")]` in grammars
- Paramatricity
  - New keyword `paramatricity x`, generates inductor
  - Formalize generating inductor from paramatricity
- Remove RuleAction, instead returning ActionResults with environments
  -> Not possible because of where the AR is allocated
- Pass flag around in context whether result of parse is needed (const bool?)
- queued_tc?
- Multi-file support
  - Make `program` part of self in TcEnv
- Add type checking to grammar adaptation
- Use a smarter data structure than a HashMap for caching
- Multi-file support
  - Grammars can be stored in `let` statement and returned from programs
    - How do manage this arena-wise?


"I found a FnConstruct with two more argument a, b"
Events are the current entries of ActionResult

"Ok, call this function on a, and this function on b please"
"When a or b is done, notify me"

Vars no longer needs to be in PR then
