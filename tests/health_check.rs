use std::net::{SocketAddr, TcpListener};

use actix_web::http::Uri;

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(uri(address, "/health_check"))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let address = spawn_app();
    let client = reqwest::Client::new();

    let response = client
        .post(uri(address, "/subscriptions"))
        .body("name=le%20guin&email=ursula_le_guin%40gmail.com")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let address = spawn_app();
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing email"),
        ("email=ursula_le_guin%40gmail.com", "missing_name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(uri(address, "/subscriptions"))
            .body(invalid_body)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}",
            error_message,
        );
    }
}

// Launch app in the background
fn spawn_app() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let address = listener
        .local_addr()
        .expect("Failed to obtain local socket address");

    let server = zero2prod::run(listener).expect("Failed to bind address");
    tokio::spawn(server);

    address
}

fn uri(address: SocketAddr, path: &str) -> String {
    Uri::builder()
        .scheme("http")
        .authority(address.to_string())
        .path_and_query(path)
        .build()
        .expect("Failed to build URI")
        .to_string()
}
