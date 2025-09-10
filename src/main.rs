use std::net::{SocketAddr, TcpListener};

use sqlx::PgPool;
use zero2prod::{get_configuration, run};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = get_configuration().expect("Failed to read configuration");
    let db_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to the Database");

    let addr = SocketAddr::from(([127, 0, 0, 1], configuration.application_port));
    let listener = TcpListener::bind(addr)?;
    println!("Listening on {addr}");

    run(listener, db_pool)?.await
}
