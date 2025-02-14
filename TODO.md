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
grammar -> adapt grammar
CheckedIndex -> DesugaredIndex
parsed value into name/grammar