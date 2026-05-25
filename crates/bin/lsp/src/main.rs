mod diagnostics;
mod state;

use std::ops::ControlFlow;

use async_lsp::{
    LanguageServer, ResponseError, client_monitor::ClientProcessMonitorLayer,
    concurrency::ConcurrencyLayer, panic::CatchUnwindLayer, router::Router, server::LifecycleLayer,
    tracing::TracingLayer,
};
use futures::future::BoxFuture;
use lmt_diagnostics::graph::ModuleGraph;
use lmt_syntax::parser::parse_program_source_with_diagnostics;
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverContents,
    HoverParams, HoverProviderCapability, InitializeParams, InitializeResult, MarkedString, OneOf,
    PublishDiagnosticsParams, ServerCapabilities, Url, notification,
};
use state::ServerState;
use tower::ServiceBuilder;
use tracing::Level;

fn publish_source_diagnostics(state: &ServerState, uri: Url, text: String) {
    let client = state.client.clone();
    let db = state.db();
    let graph = state.graph();
    let source_name = uri.to_string();

    tokio::spawn(async move {
        let module_id = {
            let mut g = graph.write();
            g.upsert(&*db, &source_name, text.clone())
        };

        let (_program, source_diagnostics) =
            parse_program_source_with_diagnostics(&text, module_id);
        let graph_guard = graph.read();
        let reports = source_diagnostics
            .into_iter()
            .map(|diag| diag.to_report(&*db, &*graph_guard))
            .collect::<Box<[_]>>();

        let lsp_diagnostics = diagnostics::to_lsp_diagnostics(&*db, &*graph_guard, &reports);
        drop(graph_guard);

        let _ = client.notify::<notification::PublishDiagnostics>(PublishDiagnosticsParams {
            uri,
            diagnostics: lsp_diagnostics,
            version: None,
        });
    });
}

/// LSP Server implementation using async-lsp Router pattern.
impl LanguageServer for ServerState {
    type Error = ResponseError;
    type NotifyResult = ControlFlow<async_lsp::Result<()>>;

    fn initialize(
        &mut self,
        _params: InitializeParams,
    ) -> BoxFuture<'static, Result<InitializeResult, Self::Error>> {
        Box::pin(async move {
            Ok(InitializeResult {
                capabilities: ServerCapabilities {
                    text_document_sync: Some(lsp_types::TextDocumentSyncCapability::Kind(
                        lsp_types::TextDocumentSyncKind::FULL,
                    )),
                    hover_provider: Some(HoverProviderCapability::Simple(true)),
                    definition_provider: Some(OneOf::Left(true)),
                    ..ServerCapabilities::default()
                },
                server_info: Some(lsp_types::ServerInfo {
                    name: "LMT Language Server".to_owned(),
                    version: Some(env!("CARGO_PKG_VERSION").to_owned()),
                }),
            })
        })
    }

    fn hover(
        &mut self,
        params: HoverParams,
    ) -> BoxFuture<'static, Result<Option<Hover>, Self::Error>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        if let Some(_text) = self.get_document(uri) {
            // TODO: Implement type inference for hover
            Box::pin(async move {
                Ok(Some(Hover {
                    contents: HoverContents::Scalar(MarkedString::String(format!(
                        "Position: line {}, col {}",
                        pos.line, pos.character
                    ))),
                    range: None,
                }))
            })
        } else {
            Box::pin(async move { Ok(None) })
        }
    }

    fn definition(
        &mut self,
        _params: GotoDefinitionParams,
    ) -> BoxFuture<'static, Result<Option<GotoDefinitionResponse>, Self::Error>> {
        // TODO: Implement goto definition
        Box::pin(async move {
            Err(ResponseError::new(
                async_lsp::ErrorCode::METHOD_NOT_FOUND,
                "Go to definition not yet implemented",
            ))
        })
    }

    fn did_open(&mut self, params: DidOpenTextDocumentParams) -> Self::NotifyResult {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text;

        // Store document
        self.insert_document(uri.clone(), text.clone());
        publish_source_diagnostics(self, uri, text);

        ControlFlow::Continue(())
    }

    fn did_change(&mut self, params: DidChangeTextDocumentParams) -> Self::NotifyResult {
        let uri = params.text_document.uri.clone();

        // Update document with full text (FULL sync)
        if let Some(change) = params.content_changes.first() {
            self.insert_document(uri.clone(), change.text.clone());
            publish_source_diagnostics(self, uri, change.text.clone());
        }

        ControlFlow::Continue(())
    }

    fn did_save(&mut self, params: DidSaveTextDocumentParams) -> Self::NotifyResult {
        let uri = params.text_document.uri;

        // Re-check and publish diagnostics on save (use stored document content)
        if let Some(text) = self.get_document(&uri) {
            publish_source_diagnostics(self, uri, text);
        }

        ControlFlow::Continue(())
    }

    fn did_close(&mut self, params: DidCloseTextDocumentParams) -> Self::NotifyResult {
        let uri = params.text_document.uri;

        // Remove document and clear diagnostics
        self.remove_document(&uri);
        let _ = self
            .client
            .notify::<notification::PublishDiagnostics>(PublishDiagnosticsParams {
                uri,
                diagnostics: Vec::new(),
                version: None,
            });

        ControlFlow::Continue(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_ansi(false)
        .with_writer(std::io::stderr)
        .init();

    let (server, _) = async_lsp::MainLoop::new_server(|client| {
        let router = Router::from_language_server(ServerState::new(client.clone()));

        // router
        //     .notification::<notification::Initialized>(|_, _| ControlFlow::Continue(()))
        //     .notification::<notification::DidChangeConfiguration>(|_, _| ControlFlow::Continue(()));

        ServiceBuilder::new()
            .layer(TracingLayer::default())
            .layer(LifecycleLayer::default())
            .layer(CatchUnwindLayer::default())
            .layer(ConcurrencyLayer::default())
            .layer(ClientProcessMonitorLayer::new(client))
            .service(router)
    });

    // Prefer truly asynchronous piped stdin/stdout without blocking tasks.
    #[cfg(unix)]
    let (stdin, stdout) = (
        async_lsp::stdio::PipeStdin::lock_tokio().unwrap(),
        async_lsp::stdio::PipeStdout::lock_tokio().unwrap(),
    );

    // Fallback to spawn blocking read/write otherwise.
    #[cfg(not(unix))]
    let (stdin, stdout) = (
        tokio_util::compat::TokioAsyncReadCompatExt::compat(tokio::io::stdin()),
        tokio_util::compat::TokioAsyncWriteCompatExt::compat_write(tokio::io::stdout()),
    );

    server.run_buffered(stdin, stdout).await.unwrap();
}
