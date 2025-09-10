use std::net::{SocketAddr, TcpListener};

use sqlx::PgPool;
use tracing::subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt};
use zero2prod::{get_configuration, run};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Set up logging
    LogTracer::init().expect("Failed to set log redirecting to tracing");

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let formatting_layer = BunyanFormattingLayer::new("zero2prod".into(), std::io::stdout);

    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);

    subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    // Read config
    let configuration = get_configuration().expect("Failed to read configuration");
    let db_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to the Database");

    // Bind listener
    let addr = SocketAddr::from(([127, 0, 0, 1], configuration.application_port));
    let listener = TcpListener::bind(addr)?;
    println!("Listening on {addr}");

    // Start server
    run(listener, db_pool)?.await
}
