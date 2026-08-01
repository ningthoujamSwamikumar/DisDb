use std::{
    error::Error,
    sync::{Arc, Mutex},
};

use dist_db::{KVStore, server::worker_orchestrator};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // initialize kv store
    let kv = Arc::new(Mutex::new(KVStore::new()));

    let address = "0.0.0.0:5555";
    let listener = TcpListener::bind(address).await.unwrap();
    println!("Listening on {address}");

    let (orchestrator_tx, orchestrator_rx) = tokio::sync::mpsc::channel::<TcpStream>(150);
    // spawn worker orchestrator
    tokio::spawn(worker_orchestrator(orchestrator_rx, kv, 10));

    loop {
        let (tcp_stream, socket_addr) = listener.accept().await?;
        println!("Connection established with {}", socket_addr);

        orchestrator_tx
            .send(tcp_stream)
            .await
            .expect("Failed to pass tcp stream to worker orchestrator!");

        println!("Waiting for another connection...");
    }
}
