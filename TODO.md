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