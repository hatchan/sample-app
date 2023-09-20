use anyhow::Result;
use clap::{Parser, Subcommand};
use opentelemetry::sdk::{trace, Resource};
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use std::process;
use tracing::{debug, error};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;
use url::Url;

mod client;
mod commands;
mod handlers;
mod models;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Application {
    #[command(subcommand)]
    command: SubCommands,

    /// Enable tracing
    #[clap(long, env)]
    tracing: bool,

    /// Endpoint of the OTLP collector
    #[clap(long, env, default_value = "http://localhost:4317")]
    otlp_endpoint: Option<Url>,
}

#[derive(Subcommand)]
enum SubCommands {
    /// Invoke a server
    Client(commands::client::Args),

    /// Start the server
    Start(commands::start::Args),
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = Application::parse();

    init_logging(&app).await;

    let result = match app.command {
        SubCommands::Client(args) => commands::client::handle_command(args).await,
        SubCommands::Start(args) => commands::start::handle_command(args).await,
    };

    match result {
        Ok(_) => debug!("Command completed successfully"),
        Err(e) => {
            error!("Command failed: {:#}", e);
            process::exit(1)
        }
    }

    Ok(())
}

async fn init_logging(app: &Application) {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    if app.tracing {
        let tracer =
            opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(
                    opentelemetry_otlp::new_exporter()
                        .tonic()
                        .with_endpoint("http://localhost:4317"),
                )
                .with_trace_config(trace::config().with_resource(Resource::new(vec![
                    KeyValue::new("service.name", "user_service"),
                ])))
                .install_batch(opentelemetry::runtime::Tokio)
                .expect("unable to install tracer");

        // This converts traces from the tracing crate to the tracer specified above.
        let otel_layer = OpenTelemetryLayer::new(tracer);

        // Finally create a subscriber and append the OpenTelemetry layer to it.
        let subscriber = subscriber.with(otel_layer);
        tracing::subscriber::set_global_default(subscriber)
            .expect("unable to set default subscriber");

        debug!("Enabling tracing support");
    } else {
        tracing::subscriber::set_global_default(subscriber)
            .expect("unable to set default subscriber");
    }
}
