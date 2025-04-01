use prism_compiler::lang::PrismDb;
use tokio::sync::RwLock;
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::lsp_types::*;
use tower_lsp_server::{Client, LanguageServer, LspService, Server};

struct Backend {
    client: Client,
    prism_env: RwLock<PrismDb>,
}

impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        self.client
            .log_message(MessageType::INFO, format!("Started Prism LSP v{VERSION}!"))
            .await;

        // Show client info
        let client_info = params.client_info.unwrap();
        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Connected to {}, version {}",
                    &client_info.name,
                    client_info.version.as_deref().unwrap_or("-")
                ),
            )
            .await;
        // self.client.log_message(MessageType::INFO, format!("Client capabilities: {:?}", params.capabilities)).await;

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "Prism LSP".to_string(),
                version: Some(VERSION.to_string()),
            }),
            capabilities: ServerCapabilities {
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            legend: SemanticTokensLegend {
                                token_types: vec![
                                    SemanticTokenType::COMMENT,
                                    // SemanticTokenType::NAMESPACE,
                                    // SemanticTokenType::TYPE,
                                    // SemanticTokenType::CLASS,
                                    // SemanticTokenType::ENUM,
                                    // SemanticTokenType::INTERFACE,
                                    // SemanticTokenType::STRUCT,
                                    // SemanticTokenType::TYPE_PARAMETER,
                                    // SemanticTokenType::PARAMETER,
                                    // SemanticTokenType::VARIABLE,
                                    // SemanticTokenType::PROPERTY,
                                    // SemanticTokenType::ENUM_MEMBER,
                                    // SemanticTokenType::EVENT,
                                    // SemanticTokenType::FUNCTION,
                                    // SemanticTokenType::METHOD,
                                    // SemanticTokenType::MACRO,
                                    // SemanticTokenType::KEYWORD,
                                    // SemanticTokenType::MODIFIER,
                                    // SemanticTokenType::STRING,
                                    // SemanticTokenType::NUMBER,
                                    // SemanticTokenType::REGEXP,
                                    // SemanticTokenType::OPERATOR,
                                    // SemanticTokenType::DECORATOR,
                                    // SemanticTokenType::new("label"),
                                ],
                                token_modifiers: vec![
                                    SemanticTokenModifier::DECLARATION,
                                    SemanticTokenModifier::DEFINITION,
                                    SemanticTokenModifier::READONLY,
                                    SemanticTokenModifier::STATIC,
                                    SemanticTokenModifier::DEPRECATED,
                                    SemanticTokenModifier::ABSTRACT,
                                    SemanticTokenModifier::ASYNC,
                                    SemanticTokenModifier::MODIFICATION,
                                    SemanticTokenModifier::DOCUMENTATION,
                                    SemanticTokenModifier::DEFAULT_LIBRARY,
                                ],
                            },
                            ..Default::default()
                        },
                    ),
                ),
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL), //TODO switch to incremental
                        ..Default::default()
                    },
                )),
                ..Default::default()
            },
        })
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        eprintln!(
            "OPEN {:?} {}",
            params.text_document.uri, params.text_document.language_id
        );
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        eprintln!("CLOSE {:?}", params.text_document.uri);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        eprintln!("CHANGE {:?}", params.text_document.uri);
        eprintln!("{:?}", params.content_changes);
    }

    async fn hover(&self, _params: HoverParams) -> Result<Option<Hover>> {
        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String("You're hovering!".to_string())),
            range: None,
        }))
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        eprintln!("TOKENS {:?}", params.text_document.uri);
        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: vec![SemanticToken {
                delta_line: 0,
                delta_start: 0,
                length: 5,
                token_type: 17,
                token_modifiers_bitset: 0,
            }],
        })))
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        prism_env: RwLock::new(PrismDb::new()),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
