// pub mod completion;
// pub mod jump_definition;
// pub mod reference;
pub mod semantic_token;
pub mod server;
// pub mod client;
// pub mod inspector;

use async_lsp::{
    concurrency::ConcurrencyLayer, panic::CatchUnwindLayer, server::LifecycleLayer,
    tracing::TracingLayer,
};
use futures::{AsyncRead, AsyncWrite};
use server::Backend;
// use tokio_with_wasm::tokio;
use tower::ServiceBuilder;
use tracing::Level;

/// # Panics
/// Panics if the server cannot be started.
#[inline]
pub async fn start(input: impl AsyncRead, output: impl AsyncWrite) {
    let (server, _) = async_lsp::MainLoop::new_server(|client| {
        // tokio::spawn({
        //     let client = client.clone();
        //     async move {
        //         let mut interval = tokio::time::interval(Duration::from_secs(1));
        //         loop {
        //             interval.tick().await;
        //             if client.emit(()).is_err() {
        //                 break;
        //             }
        //         }
        //     }
        // });

        // std::thread::spawn({
        //     let client = client.clone();
        //     move || {
        //         let mut interval = std::time::Instant::now();
        //         loop {
        //             std::thread::sleep(Duration::from_secs(1));
        //             interval += Duration::from_secs(1);
        //             if client.emit(()).is_err() {
        //                 break;
        //             }
        //         }
        //     }
        // });

        // wasm_bindgen_futures::spawn_local({
        //     let client = client.clone();
        //     async move {
        //         let mut interval = tokio::time::interval(Duration::from_secs(1));
        //         loop {
        //             interval.tick().await;
        //             if client.emit(()).is_err() {
        //                 break;
        //             }
        //         }
        //     }
        // });

        client.emit(()).unwrap();

        #[allow(unused_mut)]
        let mut builder = ServiceBuilder::new()
            .layer(TracingLayer::default())
            .layer(LifecycleLayer::default())
            .layer(CatchUnwindLayer::default())
            .layer(ConcurrencyLayer::default());

        // #[cfg(feature = "client-monitor")]
        // {
        //     builder = builder.layer(async_lsp::client_monitor::ClientProcessMonitorLayer::new(
        //         client.clone(),
        //     ));
        // }

        builder.service(Backend::new_router(client))
    });

    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_ansi(false)
        .with_writer(std::io::stderr)
        .init();

    server.run_buffered(input, output).await.unwrap();
}
