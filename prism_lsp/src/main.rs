use prism_compiler::lang::PrismDb;
use prism_input::input_table::InputTableIndex;
use prism_parser::core::tokens::{TokenType, Tokens};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::mem::take;
use std::ops::DerefMut;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;
use tower_lsp_server::{Client, LanguageServer, LspService, Server};

struct Backend {
    client: Client,
    inner: RwLock<BackendInner>,
}

#[derive(Default)]
struct BackendInner {
    db: PrismDb,
    documents: HashMap<Uri, OpenDocument>,
    document_parses: HashMap<InputTableIndex, Arc<Tokens>>,
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
}

impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        self.client
            .log_message(MessageType::INFO, format!("Starting Prism LSP v{VERSION}!"))
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
                                    SemanticTokenType::STRING,
                                    SemanticTokenType::NUMBER,
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

    async fn initialized(&self, _params: InitializedParams) {
        self.client.semantic_tokens_refresh().await.unwrap();
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

        inner.documents.insert(
            doc.uri.clone(),
            OpenDocument {
                index,
                document_type,
            },
        );

        inner.process(index, doc.uri, &self.client).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let doc = params.text_document;
        let path = PathBuf::from(doc.uri.path().as_str());
        self.client
            .log_message(MessageType::LOG, format!("UPDATE {path:?}"))
            .await;

        let mut inner = self.inner.write().await;
        let index = inner.documents[&doc.uri].index;

        let Ok([change]): std::result::Result<[_; 1], _> = params.content_changes.try_into() else {
            panic!()
        };
        assert!(change.range.is_none());
        assert!(change.range_length.is_none());

        inner.db.update_file(index, change.text);
        inner.document_parses.remove(&index);

        inner.process(index, doc.uri, &self.client).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let doc = params.text_document;
        let path = PathBuf::from(doc.uri.path().as_str());
        self.client
            .log_message(MessageType::LOG, format!("CLOSE {path:?}"))
            .await;

        let mut inner = self.inner.write().await;
        let doc = inner.documents.remove(&doc.uri).unwrap();
        inner.db.remove_file(doc.index);
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
        let mut lsp_tokens = vec![];
        {
            let mut inner = self.inner.write().await;
            let inner = inner.deref_mut();
            let index = inner.documents[&params.text_document.uri].index;

            let prism_tokens = inner.document_parses[&index].clone();

            let file_inner = inner.db.input.inner();

            let mut prev_line = 0;
            let mut prev_start = 0;

            for token in prism_tokens.to_vec() {
                // Skip empty tokens
                if file_inner
                    .slice(token.span)
                    .chars()
                    .all(|c| c.is_ascii_whitespace())
                {
                    continue;
                }

                // Convert span to LSP token info
                let (cur_line, cur_start) = file_inner.line_col_of(token.span.start_pos());
                let token_type = match token.token_type {
                    TokenType::Layout => 0,
                    TokenType::CharClass => 1,
                    TokenType::Slice => 1,
                    TokenType::Variable => 1,
                    TokenType::Keyword => 2,
                    TokenType::Symbol => 3,
                    TokenType::String => 4,
                    TokenType::Number => 5,
                };
                eprintln!("{cur_line} {cur_start} {:?}", token.token_type);

                lsp_tokens.push(SemanticToken {
                    delta_line: (cur_line - prev_line) as u32,
                    delta_start: if cur_line == prev_line {
                        cur_start - prev_start
                    } else {
                        cur_start
                    } as u32,
                    length: token.span.len() as u32,
                    token_type,
                    token_modifiers_bitset: 0,
                });

                prev_line = cur_line;
                prev_start = cur_start;
            }
        };

        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "TOKENS {}, returned {}",
                    params.text_document.uri.path().as_str(),
                    lsp_tokens.len()
                ),
            )
            .await;

        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: lsp_tokens,
        })))
    }
}

impl BackendInner {
    async fn process(&mut self, index: InputTableIndex, uri: Uri, client: &Client) {
        let (tokens, diags) = match self.documents[&uri].document_type {
            DocumentType::Prism => {
                let file = self.db.process_file(index);
                let diags = take(&mut self.db.diags);
                (file.tokens, diags)
            }
            DocumentType::PrismGrammar => {
                let (_, tokens, diags) = self.db.parse_grammar_file(index);
                (tokens, diags)
            }
        };

        // Update diagnostics
        let mut lsp_diags = Vec::new();
        {
            let file_inner = self.db.input.inner();
            for diag in diags {
                let first_span = diag.groups[0].annotations[0].span;

                let (start_line, start_char) = file_inner.line_col_of(first_span.start_pos());
                let (end_line, end_char) = file_inner.line_col_of(first_span.end_pos());

                lsp_diags.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: start_line as u32,
                            character: start_char as u32,
                        },
                        end: Position {
                            line: end_line as u32,
                            character: end_char as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: diag.title,
                    ..Diagnostic::default()
                });
            }
        }

        client
            .log_message(
                MessageType::LOG,
                format!(
                    "DIAGS {}, returned {}",
                    uri.path().as_str(),
                    lsp_diags.len()
                ),
            )
            .await;

        // Send diags to client
        client.publish_diagnostics(uri, lsp_diags, None).await;

        // Store document parse
        self.document_parses.insert(index, tokens);
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
