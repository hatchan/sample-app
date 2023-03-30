use axum::routing::{get, post};
use axum::Router;
use opentelemetry::sdk::export::trace::stdout;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::debug;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;

mod handlers;
mod models;

#[tokio::main]
async fn main() {
    init_logging().await;

    let cors = CorsLayer::very_permissive();

    // build our application with a route
    let app = Router::new()
        .route("/users/:user_name", get(handlers::get_user))
        .route("/users", post(handlers::create_user))
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn init_logging() {
    // OTEL output - stdout for now (should be jaeger or zipkin or otel collector)
    // This is part of the opentelemetry crate
    let tracer = stdout::new_pipeline()
        .with_pretty_print(true)
        .install_simple();

    // This converts traces from the tracing crate to the tracer specified above.
    let otel_layer = OpenTelemetryLayer::new(tracer);

    // Finally create a subscriber and append the OpenTelemetry layer to it.
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .finish()
        .with(otel_layer);

    tracing::subscriber::set_global_default(subscriber).expect("unable to set default subscriber");
}
