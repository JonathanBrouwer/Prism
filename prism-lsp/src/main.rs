use ariadne::Cache;
use prism_compiler::lang::PrismDb;
use prism_compiler::lang::error::PrismError;
use prism_parser::core::input_table::InputTableIndex;
use prism_parser::core::tokens::{TokenType, Tokens};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt::{Display, Formatter};
use std::ops::DerefMut;
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
    documents: HashMap<Uri, OpenDocument>,
    document_parses: HashMap<InputTableIndex, std::result::Result<Arc<Tokens>, Vec<PrismError>>>,
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
            .log_message(MessageType::INFO, format!("StartingPrism LSP v{VERSION}!"))
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
                diagnostic_provider: Some(DiagnosticServerCapabilities::RegistrationOptions(
                    DiagnosticRegistrationOptions {
                        text_document_registration_options: Default::default(),
                        diagnostic_options: DiagnosticOptions {
                            identifier: Some("prism_diagnostics".to_string()),
                            inter_file_dependencies: true,
                            workspace_diagnostics: false,
                            work_done_progress_options: Default::default(),
                        },
                        static_registration_options: StaticRegistrationOptions {
                            id: Some("prism_diagnostics".to_string()),
                        },
                    },
                )),
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

        inner.process(index).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let doc = params.text_document;
        let path = PathBuf::from(doc.uri.path().as_str());
        self.client
            .log_message(MessageType::LOG, format!("UPDATE {:?}", path))
            .await;

        let mut inner = self.inner.write().await;
        let index = inner.documents[&doc.uri].index;

        let Ok([change]): std::result::Result<[_; 1], _> = params.content_changes.try_into() else {
            panic!()
        };
        assert!(change.range.is_none());
        assert!(change.range_length.is_none());

        inner.db.input.update_file(index, change.text);
        inner.document_parses.remove(&index);

        inner.process(index).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let doc = params.text_document;
        let path = PathBuf::from(doc.uri.path().as_str());
        self.client
            .log_message(MessageType::LOG, format!("CLOSE {:?}", path))
            .await;

        let mut inner = self.inner.write().await;
        let doc = inner.documents.remove(&doc.uri).unwrap();
        inner.db.input.remove(doc.index);
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

        let mut inner = self.inner.write().await;
        let inner = inner.deref_mut();
        let index = inner.documents[&params.text_document.uri].index;

        let Ok(prism_tokens) = inner.document_parses[&index].as_ref().cloned() else {
            //TOOD tokens if error
            eprintln!("No tokens :c");
            return Ok(None);
        };

        let file_inner = inner.db.input.inner();
        let mut file_inner = &*file_inner;
        let source = (&mut file_inner).fetch(&index).unwrap();

        let mut lsp_tokens = vec![];
        let mut prev_line = 0;
        let mut prev_start = 0;

        for token in prism_tokens.to_vec() {
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
                    TokenType::Variable => 1,
                    TokenType::Keyword => 2,
                    TokenType::Symbol => 3,
                    TokenType::String => 4,
                    TokenType::Number => 5,
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

    async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> Result<DocumentDiagnosticReportResult> {
        self.client
            .log_message(
                MessageType::INFO,
                format!("DIAGS {}", params.text_document.uri.path().as_str()),
            )
            .await;

        let mut inner = self.inner.write().await;
        let inner = inner.deref_mut();
        let index = inner.documents[&params.text_document.uri].index;

        let file_inner = inner.db.input.inner();
        let mut file_inner = &*file_inner;
        let source = (&mut file_inner).fetch(&index).unwrap();

        let items: Vec<Diagnostic> = match inner.document_parses[&index] {
            Ok(ref _tokens) => {
                vec![]
            }
            Err(ref errs) => {
                let mut new_errs = vec![];
                for err in errs {
                    match err {
                        PrismError::ParseError(e) => {
                            let (_line, line, character) =
                                source.get_offset_line(e.pos.idx_in_file()).unwrap();

                            new_errs.push(Diagnostic {
                                range: Range {
                                    start: Position {
                                        line: line as u32,
                                        character: character as u32,
                                    },
                                    end: Position {
                                        line: line as u32,
                                        character: character as u32 + 1,
                                    },
                                },
                                severity: Some(DiagnosticSeverity::ERROR),
                                message: "Stuff's wrong here".to_string(),
                                ..Diagnostic::default()
                            })
                        }
                        PrismError::TypeError(e) => {}
                    }
                }

                new_errs
            }
        };

        Ok(DocumentDiagnosticReportResult::Report(
            DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
                related_documents: None,
                full_document_diagnostic_report: FullDocumentDiagnosticReport {
                    result_id: None,
                    items,
                },
            }),
        ))
    }
}

impl BackendInner {
    async fn process(&mut self, index: InputTableIndex) {
        let result = self.db.parse_grammar_file(index).map(|(_, tokens)| tokens);
        self.document_parses.insert(index, result);
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
