use std::error::Error;

use dist_db::{Frame, KVStore, KvsCommand, KvsResult, connection::Connection};
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
            let optional_frame = match connection.read_frame().await {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("Error reading frame:\n{e:#?}");
                    None
                }
            };
            match optional_frame {
                Some(frame) => {
                    println!("Received: {frame:#?}");
                    match frame {
                        Frame::Command(KvsCommand::Set { key, value }) => {
                            let res = kv.set(key, value);
                            connection.write_frame(KvsResult::Set(res).into()).await?;
                        }
                        Frame::Command(KvsCommand::Get { key }) => {
                            let res = kv.get(key).map(|r| r.to_owned());
                            connection.write_frame(KvsResult::Get(res).into()).await?;
                        }
                        _ => {
                            connection
                                .write_frame(
                                    KvsResult::Error("Invalid command!".to_string()).into(),
                                )
                                .await?;
                        }
                    }
                }
                None => break, // failed to read frame
            }
        }

        println!("Waiting for another connection...");
    }
}
