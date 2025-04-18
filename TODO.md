THE BIG TODO LIST:
- Eta reduction
- LSP support
  - Allow attributes such as `#[lsp("comment")]` in grammars
- Paramatricity
  - New keyword `paramatricity x`, generates inductor
  - Formalize generating inductor from paramatricity
- queued_tc?
- Add type checking to grammar adaptation
- Use a smarter data structure than a HashMap for caching
- Multi-file support
  - Grammars can be stored in `let` statement and returned from programs
    - How do manage this arena-wise?



TODO:
* Add `rule.group` syntax 
* Swap syntax around actions
* Switch from toxix values to max depth?

consistent name for input table
ErrorLabel optimize

blocks: &Arc<[Arc<BlockState>]>, is cloned