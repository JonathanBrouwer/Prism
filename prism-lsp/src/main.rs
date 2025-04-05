use ariadne::Cache;
use prism_compiler::lang::PrismDb;
use prism_parser::core::input_table::InputTableIndex;
use prism_parser::core::tokens::{TokenType, Tokens};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::lsp_types::*;
use tower_lsp_server::{Client, LanguageServer, LspService, Server};

struct Backend {
    client: Client,
    inner: RwLock<BackendInner>,
}

#[derive(Default)]
struct BackendInner {
    db: PrismDb,
    open_documents: HashMap<Uri, OpenDocument>,
}

#[derive(Copy, Clone)]
enum DocumentType {
    Prism,
    PrismGrammar,
}

impl Display for DocumentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DocumentType::Prism => "Prism",
                DocumentType::PrismGrammar => "Prism Grammar",
            }
        )
    }
}

struct OpenDocument {
    index: InputTableIndex,
    document_type: DocumentType,
    tokens: Arc<Tokens>,
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
                                    SemanticTokenType::VARIABLE,
                                    SemanticTokenType::KEYWORD,
                                    SemanticTokenType::OPERATOR,
                                    // SemanticTokenType::NAMESPACE,
                                    // SemanticTokenType::TYPE,
                                    // SemanticTokenType::CLASS,
                                    // SemanticTokenType::ENUM,
                                    // SemanticTokenType::INTERFACE,
                                    // SemanticTokenType::STRUCT,
                                    // SemanticTokenType::TYPE_PARAMETER,
                                    // SemanticTokenType::PARAMETER,
                                    // SemanticTokenType::PROPERTY,
                                    // SemanticTokenType::ENUM_MEMBER,
                                    // SemanticTokenType::EVENT,
                                    // SemanticTokenType::FUNCTION,
                                    // SemanticTokenType::METHOD,
                                    // SemanticTokenType::MACRO,
                                    // SemanticTokenType::MODIFIER,
                                    // SemanticTokenType::STRING,
                                    // SemanticTokenType::NUMBER,
                                    // SemanticTokenType::REGEXP,
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
        let doc = params.text_document;
        let path = PathBuf::from(doc.uri.path().as_str());
        let document_type = match doc.language_id.as_str() {
            "prism" => DocumentType::Prism,
            "prism-grammar" => DocumentType::PrismGrammar,
            lang => {
                self.client
                    .log_message(MessageType::ERROR, format!("Unknown language: {lang}"))
                    .await;
                return;
            }
        };
        self.client
            .log_message(
                MessageType::LOG,
                format!("OPEN {:?} as {}", &path, doc.language_id),
            )
            .await;

        let mut inner = self.inner.write().await;
        let index = inner.db.load_input(doc.text, path);

        let Ok((_, tokens)) = inner.db.parse_grammar_file(index) else {
            return;
        };

        inner.open_documents.insert(
            doc.uri.clone(),
            OpenDocument {
                index,
                document_type,
                tokens,
            },
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
        self.client
            .log_message(
                MessageType::INFO,
                format!("TOKENS {}", params.text_document.uri.path().as_str()),
            )
            .await;
        let inner = self.inner.read().await;

        let doc = &inner.open_documents[&params.text_document.uri];
        let file_inner = inner.db.input.inner();
        let mut file_inner = &*file_inner;
        let source = (&mut file_inner).fetch(&doc.index).unwrap();
        let prism_tokens = doc.tokens.to_vec();

        let mut lsp_tokens = vec![];
        let mut prev_line = 0;
        let mut prev_start = 0;

        for token in prism_tokens {
            let (_line, cur_line, cur_start) = source
                .get_offset_line(token.span.start_pos().idx_in_file())
                .unwrap();

            lsp_tokens.push(SemanticToken {
                delta_line: (cur_line - prev_line) as u32,
                delta_start: if cur_line == prev_line {
                    cur_start - prev_start
                } else {
                    cur_start
                } as u32,
                length: token.span.len() as u32,
                token_type: match token.token_type {
                    TokenType::CharClass => 1,
                    TokenType::Slice => 1,
                    TokenType::Keyword => 2,
                    TokenType::Symbol => 3,
                },
                token_modifiers_bitset: 0,
            });

            prev_line = cur_line;
            prev_start = cur_start;
        }

        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: lsp_tokens,
        })))
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        inner: RwLock::new(Default::default()),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
