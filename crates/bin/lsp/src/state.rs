/// LSP server state and document management.

use async_lsp::ClientSocket;
use lsp_types::Url;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// LSP server state managing open documents and diagnostics.
#[derive(Clone)]
pub struct ServerState {
    pub client: ClientSocket,
    /// Open documents: URI → text content
    pub documents: Arc<RwLock<HashMap<Url, String>>>,
}

impl ServerState {
    pub fn new(client: ClientSocket) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
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
}
