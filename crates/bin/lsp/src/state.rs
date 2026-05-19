use std::{collections::HashMap, sync::Arc};

/// LSP server state and document management.
use async_lsp::ClientSocket;
use lmt_diagnostics::graph::LspGraph;
use lmt_syntax::SyntaxDatabase;
use lsp_types::Url;
use parking_lot::RwLock;

/// LSP server state managing open documents and diagnostics.
#[derive(Clone)]
pub struct ServerState {
    pub client: ClientSocket,
    /// Open documents: URI → text content
    pub documents: Arc<RwLock<HashMap<Url, String>>>,
    /// Shared incremental database
    pub db: Arc<SyntaxDatabase>,
    /// In-memory module graph for virtual file contents
    pub graph: Arc<RwLock<LspGraph>>,
}

impl ServerState {
    pub fn new(client: ClientSocket) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
            db: Arc::new(SyntaxDatabase::new()),
            graph: Arc::new(RwLock::new(LspGraph::new())),
        }
    }

    pub fn insert_document(&self, uri: Url, content: String) {
        self.documents.write().insert(uri, content);
    }

    pub fn get_document(&self, uri: &Url) -> Option<String> {
        self.documents.read().get(uri).cloned()
    }

    pub fn remove_document(&self, uri: &Url) {
        self.documents.write().remove(uri);
    }

    pub fn db(&self) -> Arc<SyntaxDatabase> {
        self.db.clone()
    }

    pub fn graph(&self) -> Arc<RwLock<LspGraph>> {
        self.graph.clone()
    }
}
