use std::net::{SocketAddr, TcpListener};

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{address}/health_check"))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
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
