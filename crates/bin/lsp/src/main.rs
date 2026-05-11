mod client_builder;
mod client_trait;
mod diagnostics;
mod inspector;
mod server_builder;
mod server_trait;
mod state;

use std::ops::ControlFlow;

use async_lsp::{
    LanguageServer, ResponseError, client_monitor::ClientProcessMonitorLayer,
    concurrency::ConcurrencyLayer, panic::CatchUnwindLayer, router::Router, server::LifecycleLayer,
    tracing::TracingLayer,
};
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverContents,
    HoverParams, HoverProviderCapability, InitializeParams, InitializeResult, MarkedString, OneOf,
    PublishDiagnosticsParams, ServerCapabilities, notification, request,
};
use state::ServerState;
use tower::ServiceBuilder;
use tracing::Level;

/// LSP Server implementation using async-lsp Router pattern
impl LanguageServer for ServerState {
    type Error = ResponseError;
    type NotifyResult = ControlFlow<async_lsp::Result<()>>;

    fn initialize(
        &mut self,
        _params: InitializeParams,
    ) -> futures::future::BoxFuture<'static, Result<InitializeResult, Self::Error>> {
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
                    name: "LMT Language Server".to_string(),
                    version: Some(env!("CARGO_PKG_VERSION").to_string()),
                }),
            })
        })
    }

    fn hover(
        &mut self,
        params: HoverParams,
    ) -> futures::future::BoxFuture<'static, Result<Option<Hover>, Self::Error>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .clone();
        let pos = params.text_document_position_params.position;

        if let Some(_text) = self.get_document(&uri) {
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
    ) -> futures::future::BoxFuture<'static, Result<Option<GotoDefinitionResponse>, Self::Error>>
    {
        // TODO: Implement goto definition
        Box::pin(async move {
            Err(ResponseError::new(
                async_lsp::ErrorCode::METHOD_NOT_FOUND,
                "Go to definition not yet implemented",
            ))
        })
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
        let mut router = Router::from_language_server(ServerState::new(client.clone()));

        router
            .notification::<notification::DidOpenTextDocument>(|state, params| {
                let uri = params.text_document.uri.clone();
                let text = params.text_document.text.clone();

                // Store document
                state.insert_document(uri.clone(), text.clone());

                // Check and publish diagnostics
                if let Some(path) = uri
                    .to_file_path()
                    .ok()
                    .and_then(|p| p.to_str().map(String::from))
                {
                    let checker_diags = lmt_checker::check_text(&path, &text);
                    let lsp_diags = diagnostics::to_lsp_diagnostics(&text, checker_diags);

                    let client = state.client.clone();
                    client
                        .notify::<notification::PublishDiagnostics>(PublishDiagnosticsParams {
                            uri: uri.clone(),
                            diagnostics: lsp_diags,
                            version: None,
                        })
                        .unwrap();
                }

                ControlFlow::Continue(())
            })
            .notification::<notification::DidChangeTextDocument>(|state, params| {
                let uri = params.text_document.uri.clone();

                // Update document with full text (FULL sync)
                if let Some(change) = params.content_changes.first() {
                    state.insert_document(uri.clone(), change.text.clone());

                    // Check and publish diagnostics (debounced in production)
                    if let Some(path) = uri
                        .to_file_path()
                        .ok()
                        .and_then(|p| p.to_str().map(String::from))
                    {
                        let checker_diags = lmt_checker::check_text(&path, &change.text);
                        let lsp_diags =
                            diagnostics::to_lsp_diagnostics(&change.text, checker_diags);

                        let client = state.client.clone();
                        client
                            .notify::<notification::PublishDiagnostics>(PublishDiagnosticsParams {
                                uri,
                                diagnostics: lsp_diags,
                                version: None,
                            })
                            .unwrap();
                    }
                }

                ControlFlow::Continue(())
            })
            .notification::<notification::DidSaveTextDocument>(|state, params| {
                let uri = params.text_document.uri.clone();

                // Re-check and publish diagnostics on save
                if let Some(text) = state.get_document(&uri) {
                    if let Some(path) = uri
                        .to_file_path()
                        .ok()
                        .and_then(|p| p.to_str().map(String::from))
                    {
                        let checker_diags = lmt_checker::check_text(&path, &text);
                        let lsp_diags = diagnostics::to_lsp_diagnostics(&text, checker_diags);

                        let client = state.client.clone();
                        client
                            .notify::<notification::PublishDiagnostics>(PublishDiagnosticsParams {
                                uri,
                                diagnostics: lsp_diags,
                                version: None,
                            })
                            .unwrap();
                    }
                }

                ControlFlow::Continue(())
            })
            .notification::<notification::DidCloseTextDocument>(|state, params| {
                state.remove_document(&params.text_document.uri);
                ControlFlow::Continue(())
            })
            .notification::<notification::Initialized>(|_, _| ControlFlow::Continue(()))
            .notification::<notification::DidChangeConfiguration>(|_, _| ControlFlow::Continue(()));

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
