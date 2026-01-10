use prism_lsp::LspBackend;
use tower_lsp_server::{LspService, Server};

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(LspBackend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
