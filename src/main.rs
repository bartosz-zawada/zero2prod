use std::net::{SocketAddr, TcpListener};

use zero2prod::run;

const PORT: u16 = 8000;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let addr = SocketAddr::from(([127, 0, 0, 1], PORT));
    let listener = TcpListener::bind(addr)?;
    println!("Listening on {addr}");

    run(listener)?.await
}

