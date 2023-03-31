use crate::handlers;
use anyhow::{Context, Result};
use axum::routing::{get, post};
use axum::Router;
use clap::Parser;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::debug;

#[derive(Parser)]
pub struct StartArgs {
    #[clap(short, long, env, default_value = "127.0.0.1:3000")]
    listen_address: SocketAddr,
}

pub async fn handle_command(args: StartArgs) -> Result<()> {
    let cors = CorsLayer::very_permissive();

    // build our application with a route
    let app = Router::new()
        .route("/users/:user_name", get(handlers::get_user))
        .route("/users", post(handlers::create_user))
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let server = axum::Server::try_bind(&args.listen_address)
        .with_context(|| format!("failed to bind to {}", args.listen_address))?
        .serve(app.into_make_service());

    debug!("Listening on {}", server.local_addr());

    server.with_graceful_shutdown(shutdown_signal()).await?;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.unwrap();
    debug!("Received shutdown signal");
}
