use std::net::{SocketAddr, TcpListener};

use sqlx::PgPool;
use zero2prod::{get_configuration, run, telemetry};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Set up logging
    let subscriber = telemetry::get_subscriber("zero2prod", "info", std::io::stdout);
    telemetry::init_subscriber(subscriber);

    // Read config
    let configuration = get_configuration().expect("Failed to read configuration");

    // Connect to database
    let db_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to the Database");

    // Bind listener port
    let addr = SocketAddr::from(([127, 0, 0, 1], configuration.application_port));
    let listener = TcpListener::bind(addr)?;
    println!("Listening on {addr}");

    // Start server
    run(listener, db_pool)?.await
}
