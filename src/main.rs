use std::error::Error;

use dist_db::KVStore;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // initialize kv store
    let mut kv = KVStore::new();

    let address = "0.0.0.0:5555";
    let listener = TcpListener::bind(address).await.unwrap();
    println!("Listening on {address}");
    
    while let (tcp_stream, socket_addr) = listener.accept().await? {
        println!("Connection established with {}", socket_addr);

        // read loop
        loop {
            tcp_stream.readable().await?;

            // creating buf after the await prevents it from storing in the async task
            let mut buf = [0; 1024];

            match tcp_stream.try_read(&mut buf) {
                Ok(n) => {
                    println!("Read {n} bytes");
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // informed false positive readiness
                    continue;
                }
                Err(e) => {
                    e?;
                }
            }
        }
    }

    Ok(())
}
