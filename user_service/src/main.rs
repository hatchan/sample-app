use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use opentelemetry::sdk::{trace, Resource};
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use std::io;
use std::process;
use std::process::ExitCode;
use tracing::error;
use tracing::{debug, error};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, Registry};
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

    /// Log using JSON.
    #[clap(long, env = "LOG_JSON")]
    json: bool,

    /// Enable tracing
    #[clap(long, env)]
    tracing: bool,

    /// Endpoint of the OTLP collector
    #[clap(long, env, default_value = "http://localhost:4317")]
    otlp_endpoint: Url,
}

#[derive(Subcommand)]
enum SubCommands {
    /// Invoke a server
    Client(commands::client::Args),

    /// Start the server
    Start(commands::start::Args),
}

#[tokio::main]
async fn main() -> ExitCode {
    let app = Application::parse();

    let result = init_logging(&app);
    if let Err(err) = result {
        error!(%err, "unable to initialize logging");
        return ExitCode::FAILURE;
    }

    let result = match app.command {
        SubCommands::Client(args) => commands::client::handle_command(args).await,
        SubCommands::Start(args) => commands::start::handle_command(args).await,
    };

    if let Err(e) = result {
        error!("Unable to initialize logging: {:#}", e);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn init_logging(app: &Application) -> Result<()> {
    // The filter layer controls which log levels to display.
    let filter_layer = EnvFilter::from_default_env();

    // The log layer controls the output of log events to stderr. Depending on the
    // `json` flag, it will either be human readable or json encoded.
    let log_layer = tracing_subscriber::fmt::layer().with_writer(io::stderr);
    let log_layer = if app.json {
        log_layer.json().boxed()
    } else {
        log_layer.boxed()
    };

    // The trace layer will send traces to the configured tracing backend
    // depending on the `tracing` flag.
    let trace_layer = if app.tracing {
        // This tracer is responsible for sending the actual traces.
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(app.otlp_endpoint.to_string()),
            )
            .with_trace_config(
                trace::config()
                    .with_resource(Resource::new(vec![KeyValue::new("service.name", "api")])),
            )
            .install_batch(opentelemetry::runtime::Tokio)
            .context("unable to install tracer: {err}")?;

        // This layer will take the traces from the `tracing` crate and send
        // them to the tracer specified above.
        Some(OpenTelemetryLayer::new(tracer))
    } else {
        None
    };

    Registry::default()
        .with(filter_layer)
        .with(log_layer)
        .with(trace_layer)
        .try_init()
        .context("unable to initialize logger")?;

    Ok(())
}
