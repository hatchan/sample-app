use crate::handlers;
use anyhow::{Context, Result};
use axum::body::HttpBody;
use axum::response::Response;
use axum::routing::{get, post};
use axum::Router;
use clap::Parser;
use http::{Request, StatusCode};
use std::error::Error;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::Poll;
use tower::{Layer, Service};
use tower_http::cors::CorsLayer;
use tracing::debug;

#[derive(Parser)]
pub struct Args {
    #[clap(short, long, env, default_value = "127.0.0.1:3000")]
    listen_address: SocketAddr,
}

pub async fn handle_command(args: Args) -> Result<()> {
    // build our application with a route
    let app = Router::new()
        .route("/users/:user_name", get(handlers::get_user))
        .route("/users", post(handlers::create_user))
        .layer(CorsLayer::very_permissive())
        // .layer(TraceLayer::new_for_http())
        .layer(OtlpLayer::new());

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

#[derive(Debug, Clone)]
pub struct Otlp<S> {
    inner: S,
}

impl<S> Otlp<S> {
    fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<T, Request> Service<Request> for Otlp<T>
where
    T: Service<Request>,
    T::Future: 'static,
    T::Error: Into<Box<dyn Error + Send + Sync>> + 'static,
    T::Response: 'static,
{
    type Response = T::Response;
    type Error = Box<dyn Error + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let fut = self.inner.call(req);

        // create a response in a future.
        let f = async move {
            let resp = fut;

            resp.await.map_err(|err| err.into())
        };

        // Return the response as an immediate future
        Box::pin(f)
    }
}

#[derive(Clone)]
pub struct OtlpLayer {}

impl OtlpLayer {
    pub fn new() -> Self {
        Self {}
    }
}

impl<S> Layer<S> for OtlpLayer {
    type Service = Otlp<S>;

    fn layer(&self, service: S) -> Self::Service {
        Otlp::new(service)
    }
}
