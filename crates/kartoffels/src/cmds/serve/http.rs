use anyhow::Result;
use axum::extract::ws::Message;
use axum::extract::{DefaultBodyLimit, State as AxumState, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use futures_util::{SinkExt, StreamExt};
use glam::uvec2;
use kartoffels_store::Store;
use kartoffels_ui::{Term, TermType};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tower_http::cors::{self, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::info;

pub async fn start(
    addr: &SocketAddr,
    store: Arc<Store>,
    shutdown: CancellationToken,
) -> Result<()> {
    info!(?addr, "starting http server");

    let listener = TcpListener::bind(&addr).await?;

    let app = {
        let cors = CorsLayer::new()
            .allow_methods(cors::Any)
            .allow_headers(cors::Any)
            .allow_origin(cors::Any);

        let trace = TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::default());

        let limit = DefaultBodyLimit::max(512 * 1024);

        Router::new()
            .route("/", get(handle_connect))
            .with_state(store)
            .layer(cors)
            .layer(trace)
            .layer(limit)
    };

    info!("ready");

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown.cancelled_owned())
        .await?;

    Ok(())
}

async fn handle_connect(
    AxumState(store): AxumState<Arc<Store>>,
    socket: WebSocketUpgrade,
) -> impl IntoResponse {
    // We already buffer our stdout by relying on Ratatui, there's no need for
    // extra buffering on the socket's side
    let socket = socket.write_buffer_size(0);

    // We need ~256 Kb for *.elf file upload (128 Kb, but base64'd), let's round
    // to 512 Kb for good measure
    let socket = socket.max_message_size(512 * 1024);

    socket.on_upgrade(|socket| async move {
        info!("connection opened");

        let (stdout, stdin) = socket.split();

        let stdin = stdin.filter_map(|msg| async move {
            match msg {
                Ok(Message::Text(msg)) => Some(Ok(msg.into_bytes())),
                Ok(Message::Binary(msg)) => Some(Ok(msg)),
                Ok(_) => None,
                Err(err) => Some(Err(err.into())),
            }
        });

        let stdout =
            stdout.with(|stdout| async move { Ok(Message::Binary(stdout)) });

        let stdin = Box::pin(stdin);
        let stdout = Box::pin(stdout);
        let size = uvec2(0, 0);

        let mut term = Term::new(TermType::Http, stdin, stdout, size)
            .await
            .unwrap();

        match kartoffels_ui::start(&mut term, &store).await {
            Ok(()) => {
                info!("connection closed");
            }
            Err(err) => {
                info!("connection closed: {:?}", err);
            }
        }
    })
}
