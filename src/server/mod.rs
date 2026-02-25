pub mod routes;

use anyhow::{Context, Result};
use tokio::net::TcpListener;
use tokio::signal;
use tracing_subscriber::EnvFilter;

use crate::cli::ServerOpts;
use crate::db;
use crate::state::AppState;

/// Initialize tracing and create + run the tokio runtime.
/// `foreground`: true = log to stdout, false = log to file (daemon mode).
pub fn run(opts: ServerOpts, foreground: bool) -> Result<()> {
    init_tracing(&opts.log_level, foreground);

    let rt = tokio::runtime::Runtime::new().context("failed to create tokio runtime")?;
    rt.block_on(run_server(opts))
}

fn init_tracing(log_level: &str, foreground: bool) {
    let filter = EnvFilter::try_new(log_level).unwrap_or_else(|_| EnvFilter::new("info"));

    if foreground {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_ansi(false)
            .init();
    }
}

async fn run_server(opts: ServerOpts) -> Result<()> {
    let data_dir = &opts.data_dir;
    std::fs::create_dir_all(data_dir)?;
    std::fs::create_dir_all(data_dir.join("artifacts"))?;

    let pool = db::create_pool(data_dir).await?;

    let state = AppState {
        db: pool,
        data_dir: data_dir.clone(),
        max_upload_size: opts.max_upload_size,
    };

    let app = routes::router(state);

    let addr = format!("{}:{}", opts.host, opts.port);
    let listener = TcpListener::bind(&addr)
        .await
        .with_context(|| format!("failed to bind to {}", addr))?;

    tracing::info!("cask listening on {}", addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .context("server error")?;

    tracing::info!("cask shut down gracefully");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to listen for ctrl+c");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to listen for SIGTERM")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
