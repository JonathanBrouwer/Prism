use crate::{DocumentType, LspBackend, LspBackendInner, OpenDocument};
use prism_compiler::parser::lexer::Token;
use prism_input::input_table::{InputTableIndex, InputTableInner};
use prism_input::span::Span;
use std::mem::take;
use std::ops::DerefMut;
use std::path::PathBuf;
use tower_lsp_server::ls_types::{
    Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, DidChangeTextDocumentParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, Hover, HoverParams,
    HoverProviderCapability, InitializeParams, InitializeResult, InitializedParams, Location,
    MessageType, Position, Range, SemanticToken, SemanticTokenModifier, SemanticTokenType,
    SemanticTokens, SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions,
    SemanticTokensParams, SemanticTokensResult, SemanticTokensServerCapabilities,
    ServerCapabilities, ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind,
    TextDocumentSyncOptions, Uri,
};
use tower_lsp_server::{Client, LanguageServer};

impl LanguageServer for LspBackend {
    async fn initialize(
        &self,
        params: InitializeParams,
    ) -> tower_lsp_server::jsonrpc::Result<InitializeResult> {
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
                                    SemanticTokenType::PARAMETER,
                                    // SemanticTokenType::NAMESPACE,
                                    // SemanticTokenType::TYPE,
                                    // SemanticTokenType::CLASS,
                                    // SemanticTokenType::ENUM,
                                    // SemanticTokenType::INTERFACE,
                                    // SemanticTokenType::STRUCT,
                                    // SemanticTokenType::TYPE_PARAMETER,
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

    async fn shutdown(&self) -> tower_lsp_server::jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let doc = params.text_document;
        let path = PathBuf::from(doc.uri.path().as_str());
        let document_type = match doc.language_id.as_str() {
            "prism" => DocumentType::Prism,
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

    async fn hover(
        &self,
        _params: HoverParams,
    ) -> tower_lsp_server::jsonrpc::Result<Option<Hover>> {
        Ok(None)
        // Ok(Some(Hover {
        //     contents: HoverContents::Scalar(MarkedString::String("You're hovering!".to_string())),
        //     range: None,
        // }))
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> tower_lsp_server::jsonrpc::Result<Option<SemanticTokensResult>> {
        self.client
            .log_message(
                MessageType::INFO,
                format!("TOKENS {}...", params.text_document.uri.path().as_str(),),
            )
            .await;

        let mut lsp_tokens = vec![];
        {
            let mut inner = self.inner.write().await;
            let inner = inner.deref_mut();
            let index = inner.documents[&params.text_document.uri].index;

            let prism_tokens = inner.document_parses[&index].clone();

            let file_inner = inner.db.input.inner();

            let mut prev_line = 0;
            let mut prev_start = 0;

            for token in &*prism_tokens {
                // Convert span to LSP token info
                let (cur_line, cur_start) = file_inner.line_col_of(token.span().start_pos());
                let token_type = match token {
                    Token::Newline(_) => continue,
                    Token::Whitespace(_) => continue,
                    Token::Comment(_) => 0,
                    Token::OpenParen(_) => continue,
                    Token::CloseParen(_) => continue,
                    Token::Identifier(_) => 6,
                    Token::Keyword(_) => 2,
                    Token::Symbol(_) => 3,
                    Token::StringLit(_) => 4,
                    Token::NumLit(_) => 5,
                };

                lsp_tokens.push(SemanticToken {
                    delta_line: (cur_line - prev_line) as u32,
                    delta_start: if cur_line == prev_line {
                        cur_start - prev_start
                    } else {
                        cur_start
                    } as u32,
                    length: token.span().len() as u32,
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

impl LspBackendInner {
    async fn process(&mut self, index: InputTableIndex, uri: Uri, client: &Client) {
        let (tokens, diags) = match self.documents[&uri].document_type {
            DocumentType::Prism => {
                let file = self.db.process_file(index);
                let diags = take(&mut self.db.diags);
                (file.tokens, diags)
            }
        };

        // Update diagnostics
        let mut lsp_diags = Vec::new();
        {
            let input = self.db.input.inner();
            for diag in diags {
                let first_span = diag.groups[0].annotations[0].span;

                let related_information = diag
                    .groups
                    .iter()
                    .flat_map(|group| {
                        group
                            .annotations
                            .iter()
                            .map(|annot| DiagnosticRelatedInformation {
                                location: Location {
                                    uri: Uri::from_file_path(
                                        input.get_path(annot.span.start_pos().file()),
                                    )
                                    .unwrap(),
                                    range: Self::span_to_range(&input, annot.span),
                                },
                                message: match annot.label.as_ref() {
                                    Some(label) => label.to_string(),
                                    None => "<no label>".to_string(),
                                },
                            })
                    })
                    .collect();

                lsp_diags.push(Diagnostic {
                    range: Self::span_to_range(&input, first_span),
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: diag.title,
                    related_information: Some(related_information),
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

    fn span_to_range(input: &InputTableInner, span: Span) -> Range {
        let (start_line, start_char) = input.line_col_of(span.start_pos());
        let (end_line, end_char) = input.line_col_of(span.end_pos());
        Range {
            start: Position {
                line: start_line as u32,
                character: start_char as u32,
            },
            end: Position {
                line: end_line as u32,
                character: end_char as u32,
            },
        }
    }
}
