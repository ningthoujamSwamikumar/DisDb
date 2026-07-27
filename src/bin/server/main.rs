use std::{error::Error, str::from_utf8};

use dist_db::{KVStore, connection::Connection};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // initialize kv store
    let mut kv = KVStore::new();

    let address = "0.0.0.0:5555";
    let listener = TcpListener::bind(address).await.unwrap();
    println!("Listening on {address}");

    loop {
        let (tcp_stream, socket_addr) = listener.accept().await?;

        println!("Connection established with {}", socket_addr);

        let mut connection = Connection::new(tcp_stream);

        // loops frames until connection is dropped
        loop {
            match connection.read_frame().await? {
                Some(frame) => {
                    println!("Received: {frame:#?}");
                    match frame {
                        dist_db::KvsCommand::Set { key, value } => {
                            let res = kv.set(key, value);
                            connection.write_frame(dist_db::KvsResult::Set(res)).await?;
                        }
                        dist_db::KvsCommand::Get { key } => {
                            let res = kv.get(key).map(|r| r.to_owned());
                            connection.write_frame(dist_db::KvsResult::Get(res)).await?;
                        }
                    }
                }
                None => break, // connection is closed
            }
        }

        println!("Waiting for another connection...");
    }
}
