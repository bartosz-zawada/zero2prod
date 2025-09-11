use std::{
    net::{SocketAddr, TcpListener},
    sync::LazyLock,
};

use actix_web::http::Uri;
use secrecy::ExposeSecret;
use sqlx::{Connection, PgConnection, PgPool};
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use uuid::Uuid;
use zero2prod::{
    DatabaseSettings, get_configuration,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    // Send request to API
    let response = client
        .get(app.uri_for("/health_check"))
        .send()
        .await
        .expect("Failed to execute request");

    // Check API response
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Send request to API
    let response = client
        .post(app.uri_for("/subscriptions"))
        .body("name=le%20guin&email=ursula_le_guin%40gmail.com")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await
        .expect("Failed to execute request");

    // Check API response
    assert_eq!(200, response.status().as_u16());

    // Verify changes to DB
    let saved = sqlx::query!("SELECT email, person_name FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.person_name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing email"),
        ("email=ursula_le_guin%40gmail.com", "missing_name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Send request to API
        let response = client
            .post(app.uri_for("/subscriptions"))
            .body(invalid_body)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()
            .await
            .expect("Failed to execute request");

        // Check API response
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}",
            error_message,
        );
    }
}

static TRACING: LazyLock<()> = LazyLock::new(|| {
    let sink = if std::env::var_os("TEST_LOG").is_some() {
        BoxMakeWriter::new(std::io::stdout)
    } else {
        BoxMakeWriter::new(std::io::sink)
    };

    let subscriber = get_subscriber("test", "debug", sink);
    init_subscriber(subscriber);
});

struct TestApp {
    address: SocketAddr,
    db_pool: PgPool,
}

impl TestApp {
    fn uri_for(&self, path: &str) -> String {
        Uri::builder()
            .scheme("http")
            .authority(self.address.to_string())
            .path_and_query(path)
            .build()
            .expect("Failed to build URI")
            .to_string()
    }
}

async fn configure_database(mut config: DatabaseSettings) -> PgPool {
    config.database_name = format!("test_{}", Uuid::new_v4());

    // Create database
    let maintenance_settings = DatabaseSettings {
        database_name: "postgres".to_string(),
        username: "postgres".to_string(),
        password: "password".into(),
        host: config.host.clone(),
        port: config.port,
    };

    let mut connection =
        PgConnection::connect(maintenance_settings.connection_string().expose_secret())
            .await
            .expect("Failed to connect to Postgres");

    sqlx::query(
        format!(
            "CREATE DATABASE \"{}\" WITH OWNER \"{}\";",
            config.database_name, config.username,
        )
        .as_str(),
    )
    .execute(&mut connection)
    .await
    .expect("Failed to create database");

    // Migrate database
    let connection_pool = PgPool::connect(config.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

// Launch app in the background
async fn spawn_app() -> TestApp {
    // Enable logging in test
    LazyLock::force(&TRACING);

    // Bind a random port
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let address = listener
        .local_addr()
        .expect("Failed to obtain local socket address");

    // Read config
    let config = get_configuration().expect("Failed to read configuration");

    // Set up database
    let db_pool = configure_database(config.database).await;

    // Run server as a separate task
    let server = zero2prod::run(listener, db_pool.clone()).expect("Failed to bind address");
    tokio::spawn(server);

    // Return context information
    TestApp { address, db_pool }
}
