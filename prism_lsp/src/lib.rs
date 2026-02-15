mod language_server;

use prism_compiler::lang::PrismDb;
use prism_input::input_table::InputTableIndex;
use prism_input::tokens::Tokens;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp_server::Client;
use tower_lsp_server::ls_types::*;

pub struct LspBackend {
    client: Client,
    inner: RwLock<LspBackendInner>,
}

impl LspBackend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            inner: Default::default(),
        }
    }
}

#[derive(Default)]
struct LspBackendInner {
    db: PrismDb,
    documents: HashMap<Uri, OpenDocument>,
    document_parses: HashMap<InputTableIndex, Arc<Tokens>>,
}

#[derive(Copy, Clone)]
enum DocumentType {
    Prism,
}

impl Display for DocumentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DocumentType::Prism => "Prism",
            }
        )
    }
}

struct OpenDocument {
    index: InputTableIndex,
    document_type: DocumentType,
}
