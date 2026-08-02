use std::{
    error::Error,
    sync::{Arc, Mutex},
};

use dist_db::{KVStore, server::Server};
use tokio::net::{TcpListener};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // initialize kv store
    let kv = Arc::new(Mutex::new(KVStore::new()));

    let address = "0.0.0.0:5555";
    let listener = TcpListener::bind(address).await.unwrap();
    println!("Listening on {address}");

    let server = Server::new(kv);

    loop {
        let (tcp_stream, socket_addr) = listener.accept().await?;
        println!("Connection established with {}", socket_addr);

        server.run(tcp_stream).await;

        println!("Waiting for another connection...");
    }
}
